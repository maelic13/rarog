# Vendored Fathom

Rarog builds the Syzygy probing library from the vendored Fathom C sources
under `src/` (`vendor/fathom/LICENSE`, MIT).

## Local patches

Apply local patches in order when refreshing from upstream Fathom:

- `patches/01-utf8-windows-open.patch`: converts UTF-8 `SyzygyPath` filenames
  to UTF-16 via `MultiByteToWideChar(CP_UTF8, ...)` and calls `CreateFileW` in
  `open_tb` on Windows, instead of relying on `CreateFileA` or `mbstowcs_s`
  under the process ANSI code page. Reapply with:

  ```sh
  git apply vendor/fathom/patches/01-utf8-windows-open.patch
  ```
