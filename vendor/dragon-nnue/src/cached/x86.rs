//! Integer reductions may reassociate modulo 2^32; float reductions may not.
use crate::{Head, LANES};
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
pub(super) unsafe fn update(
    acc: &mut [i16; LANES],
    weights: &[i16],
    removed: &[usize],
    added: &[usize],
) {
    if removed.is_empty() && added.is_empty() {
        return;
    }
    let mut rows = [std::ptr::null(); 64];
    for (slot, &index) in rows.iter_mut().zip(removed.iter().chain(added)) {
        *slot = weights[index * LANES..(index + 1) * LANES].as_ptr();
    }
    // Fixed-size common deltas avoid two variable-length loops per vector.
    // SAFETY: Row pointers were bounds checked above and counts match each arm.
    unsafe {
        match (removed.len(), added.len()) {
            (1, 1) => return update_small::<1, 1>(acc, &rows),
            (2, 1) => return update_small::<2, 1>(acc, &rows),
            (1, 2) => return update_small::<1, 2>(acc, &rows),
            (2, 2) => return update_small::<2, 2>(acc, &rows),
            (3, 2) => return update_small::<3, 2>(acc, &rows),
            (2, 3) => return update_small::<2, 3>(acc, &rows),
            _ => {}
        }
    }
    // SAFETY: Every row was bounds checked above. LANES is divisible by 16;
    // unaligned 32-byte accesses stay within each 1536-element row and acc.
    unsafe {
        for i in (0..LANES).step_by(16) {
            let mut sum = _mm256_loadu_si256(acc.as_ptr().add(i).cast());
            for &row in &rows[..removed.len()] {
                sum = _mm256_sub_epi16(sum, _mm256_loadu_si256(row.add(i).cast()));
            }
            for &row in &rows[removed.len()..removed.len() + added.len()] {
                sum = _mm256_add_epi16(sum, _mm256_loadu_si256(row.add(i).cast()));
            }
            _mm256_storeu_si256(acc.as_mut_ptr().add(i).cast(), sum);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn update_small<const R: usize, const A: usize>(
    acc: &mut [i16; LANES],
    rows: &[*const i16; 64],
) {
    // SAFETY: The caller supplies R+A valid rows; 16-lane accesses cover
    // exactly LANES elements. Constant bounds let LLVM unroll the row loops.
    unsafe {
        for i in (0..LANES).step_by(16) {
            let mut sum = _mm256_loadu_si256(acc.as_ptr().add(i).cast());
            for &row in &rows[..R] {
                sum = _mm256_sub_epi16(sum, _mm256_loadu_si256(row.add(i).cast()));
            }
            for &row in &rows[R..R + A] {
                sum = _mm256_add_epi16(sum, _mm256_loadu_si256(row.add(i).cast()));
            }
            _mm256_storeu_si256(acc.as_mut_ptr().add(i).cast(), sum);
        }
    }
}

#[target_feature(enable = "avx2")]
pub(super) unsafe fn activate(acc: &[i16; LANES], output: &mut [u8]) {
    assert_eq!(output.len(), 768);
    let zero = _mm256_setzero_si256();
    let max = _mm256_set1_epi16(127);
    // SAFETY: 48 blocks cover each 768-element half; each output block writes 16 bytes.
    unsafe {
        for i in (0..768).step_by(16) {
            let a = _mm256_min_epi16(
                _mm256_max_epi16(_mm256_loadu_si256(acc.as_ptr().add(i).cast()), zero),
                max,
            );
            let b = _mm256_min_epi16(
                _mm256_max_epi16(_mm256_loadu_si256(acc.as_ptr().add(i + 768).cast()), zero),
                max,
            );
            let product = _mm256_srli_epi16::<7>(_mm256_mullo_epi16(a, b));
            let packed = _mm256_permute4x64_epi64::<0xd8>(_mm256_packus_epi16(product, zero));
            _mm_storeu_si128(
                output.as_mut_ptr().add(i).cast(),
                _mm256_castsi256_si128(packed),
            );
        }
    }
}

macro_rules! first_layer {
    ($name:ident, $features:literal, $dot:ident) => {
        #[target_feature(enable = $features)]
        unsafe fn $name(head: &Head, input: &[u8; LANES], first: &mut [f32; 16]) {
            // SAFETY: All 16 rows contain LANES bytes; 32-byte blocks cover
            // each row and the input exactly. Kernel dispatch checked the ISA.
            unsafe {
                for group in (0..16).step_by(4) {
                    let mut sums = [_mm256_setzero_si256(); 4];
                    for i in (0..LANES).step_by(32) {
                        let a = _mm256_loadu_si256(input.as_ptr().add(i).cast());
                        for (row, sum) in sums.iter_mut().enumerate() {
                            let w = _mm256_loadu_si256(
                                head.weights1.as_ptr().add((group + row) * LANES + i).cast(),
                            );
                            *sum = $dot(*sum, a, w);
                        }
                    }
                    for (row, sum) in sums.into_iter().enumerate() {
                        let mut lanes = [0_i32; 8];
                        _mm256_storeu_si256(lanes.as_mut_ptr().cast(), sum);
                        let dot = lanes
                            .into_iter()
                            .fold(head.bias1[group + row], i32::wrapping_add);
                        first[group + row] = dot as f32;
                    }
                }
            }
        }
    };
}

#[target_feature(enable = "avx2")]
unsafe fn dot_avx2(sum: __m256i, a: __m256i, w: __m256i) -> __m256i {
    _mm256_add_epi32(
        sum,
        _mm256_madd_epi16(_mm256_maddubs_epi16(a, w), _mm256_set1_epi16(1)),
    )
}

#[target_feature(enable = "avx2,avxvnni")]
unsafe fn dot_vnni(sum: __m256i, a: __m256i, w: __m256i) -> __m256i {
    // SCReLU inputs <=126: adjacent pairs cannot saturate i16. VNNI's
    // four-product sum is therefore identical to two reference pair sums.
    _mm256_dpbusd_avx_epi32(sum, a, w)
}

first_layer!(first_avx2, "avx2", dot_avx2);

first_layer!(first_vnni, "avx2,avxvnni", dot_vnni);

#[target_feature(enable = "avx2")]
pub(super) unsafe fn forward_avx2(head: &Head, input: &[u8; LANES]) -> i32 {
    let mut first = [0_f32; 16];
    // SAFETY: This function requires AVX2, as do both called kernels.
    unsafe {
        first_avx2(head, input, &mut first);
        finish(head, &first)
    }
}

#[target_feature(enable = "avx2,avxvnni")]
pub(super) unsafe fn forward_vnni(head: &Head, input: &[u8; LANES]) -> i32 {
    let mut first = [0_f32; 16];
    // SAFETY: This function requires both ISAs used by the called kernels.
    unsafe {
        first_vnni(head, input, &mut first);
        finish(head, &first)
    }
}

#[target_feature(enable = "avx2")]
unsafe fn finish(head: &Head, first: &[f32; 16]) -> i32 {
    let skip = (first[15] as i32).wrapping_mul(9600) / 8128;
    let mut activated = [0_f32; 30];
    for i in 0..15 {
        activated[i] = (first[i] * first[i] * (1.0 / 524_288.0)).min(127.0);
        activated[i + 15] = (first[i].max(0.0) * (1.0 / 64.0)).min(127.0);
    }
    let mut second = [0_f32; 32];
    // SAFETY: This function and the called kernel both require AVX2.
    unsafe {
        second_layer(head, &activated, &mut second);
    }
    head.output(skip, &second)
}

#[target_feature(enable = "avx2")]
unsafe fn second_layer(head: &Head, activated: &[f32; 30], second: &mut [f32; 32]) {
    // Vectorize across output neurons, retaining each neuron's original four
    // accumulation lanes, reduction tree and separate multiply/add rounding.
    // SAFETY: Each transposed row has 32 floats; groups cover exactly eight.
    unsafe {
        for group in (0..32).step_by(8) {
            let mut sums = [_mm256_setzero_ps(); 4];
            for i in (0..28).step_by(4) {
                for lane in 0..4 {
                    let w = _mm256_loadu_ps(head.weights2_transposed[i + lane].as_ptr().add(group));
                    sums[lane] = _mm256_add_ps(
                        sums[lane],
                        _mm256_mul_ps(w, _mm256_set1_ps(activated[i + lane])),
                    );
                }
            }
            let reduced = _mm256_add_ps(
                _mm256_add_ps(sums[0], sums[2]),
                _mm256_add_ps(sums[1], sums[3]),
            );
            let tail = _mm256_add_ps(
                _mm256_mul_ps(
                    _mm256_loadu_ps(head.weights2_transposed[28].as_ptr().add(group)),
                    _mm256_set1_ps(activated[28]),
                ),
                _mm256_mul_ps(
                    _mm256_loadu_ps(head.weights2_transposed[29].as_ptr().add(group)),
                    _mm256_set1_ps(activated[29]),
                ),
            );
            let dot = _mm256_add_ps(tail, reduced);
            let value = _mm256_min_ps(
                _mm256_mul_ps(
                    _mm256_max_ps(
                        _mm256_add_ps(dot, _mm256_loadu_ps(head.bias2.as_ptr().add(group))),
                        _mm256_setzero_ps(),
                    ),
                    _mm256_set1_ps(1.0 / 64.0),
                ),
                _mm256_set1_ps(127.0),
            );
            _mm256_storeu_ps(second.as_mut_ptr().add(group), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_intermediates_preserve_integer_wrap_and_float_bits() {
        if !std::is_x86_feature_detected!("avx2") {
            return;
        }
        let weights2 = std::array::from_fn(|r| {
            std::array::from_fn(|c| ((r * 17 + c * 31) % 256) as i8 as f32)
        });
        let weights1: Vec<i8> = (0..16 * LANES).map(|i| (i * 137) as i8).collect();
        let head = Head {
            bias1: std::array::from_fn(|r| {
                if r % 2 == 0 {
                    i32::MAX - 17
                } else {
                    i32::MIN + 23
                }
            }),
            weights1,
            bias2: std::array::from_fn(|i| i as f32 * 13.0 - 100.0),
            weights2_transposed: std::array::from_fn(|i| std::array::from_fn(|j| weights2[j][i])),
            weights2,
            bias_out: 0.0,
            weights_out: [1.0; 32],
        };
        for pattern in 0..16 {
            let input = std::array::from_fn(|i| match pattern {
                0 => 0,
                1 => 126,
                _ => ((i * 37 + pattern * 19) % 127) as u8,
            });
            let mut expected = [0.0_f32; 16];
            for (r, value) in expected.iter_mut().enumerate() {
                let mut sum = head.bias1[r];
                for i in (0..LANES).step_by(2) {
                    let pair = i32::from(input[i]) * i32::from(head.weights1[r * LANES + i])
                        + i32::from(input[i + 1]) * i32::from(head.weights1[r * LANES + i + 1]);
                    sum = sum.wrapping_add(pair.clamp(-32768, 32767));
                }
                *value = sum as f32;
            }
            let mut actual = [0.0; 16];
            // SAFETY: The test checks AVX2 support before executing kernels.
            unsafe {
                first_avx2(&head, &input, &mut actual);
            }
            assert_eq!(actual.map(f32::to_bits), expected.map(f32::to_bits));
            if std::is_x86_feature_detected!("avxvnni") {
                // SAFETY: The test checked both AVX2 and VNNI support.
                unsafe {
                    first_vnni(&head, &input, &mut actual);
                }
                assert_eq!(actual.map(f32::to_bits), expected.map(f32::to_bits));
            }
            let activated =
                std::array::from_fn(|i| ((i * 71 + pattern * 17) % 16384) as f32 / 128.0);
            let mut expected = [0_f32; 32];
            for (r, value) in expected.iter_mut().enumerate() {
                let mut lanes = [0.0; 4];
                for i in (0..28).step_by(4) {
                    for lane in 0..4 {
                        lanes[lane] += head.weights2[r][i + lane] * activated[i + lane];
                    }
                }
                let reduced = (lanes[0] + lanes[2]) + (lanes[1] + lanes[3]);
                let tail =
                    head.weights2[r][28] * activated[28] + head.weights2[r][29] * activated[29];
                *value = ((tail + reduced + head.bias2[r]).max(0.0) / 64.0).min(127.0);
            }
            let mut actual = [0.0; 32];
            // SAFETY: The test checked AVX2 support.
            unsafe {
                second_layer(&head, &activated, &mut actual);
            }
            assert_eq!(actual.map(f32::to_bits), expected.map(f32::to_bits));
        }
    }
}
