# Third-Party Software

This application bundles the following third-party software:

## mpv / libmpv
Licensed under the GNU Lesser General Public License v2.1 or later (LGPLv2.1+).
Source: https://github.com/mpv-player/mpv
Windows build: https://github.com/zhongfly/mpv-winbuild
Android build: https://github.com/jarnedemeulemeester/libmpv-android
iOS build: https://github.com/mpvkit/MPVKit (LGPL variant)
macOS build: Homebrew's mpv (https://formulae.brew.sh/formula/mpv), which is built with GPL components and is therefore distributed under the GNU GPL v3.
Full license text: https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html

## FFmpeg
Licensed under the GNU Lesser General Public License v2.1 or later (LGPLv2.1+).
Source: https://ffmpeg.org
Full license text: https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html

## iOS libraries (via MPVKit)
On iOS, libmpv and FFmpeg are statically linked together with the libraries below. Hoshi's complete source code is available under the AGPL-3.0, so the app can be rebuilt and relinked against modified versions of the LGPL libraries.

| Library | License |
| ------- | ------- |
| libass | ISC |
| FreeType | FreeType License (FTL) |
| FriBidi | LGPLv2.1+ |
| HarfBuzz | MIT |
| libunibreak | zlib |
| libplacebo | LGPLv2.1+ |
| libdovi | MIT |
| Little CMS (lcms2) | MIT |
| shaderc | Apache-2.0 |
| MoltenVK | Apache-2.0 |
| dav1d | BSD-2-Clause |
| uavs3d | BSD-3-Clause |
| libbluray | LGPLv2.1+ |
| uchardet | MPL-1.1 / GPLv2+ / LGPLv2.1+ |
| GnuTLS | LGPLv2.1+ |
| Nettle / Hogweed | LGPLv3+ or GPLv2+ |
| GMP | LGPLv3+ or GPLv2+ |
| OpenSSL 3 | Apache-2.0 |

Sources and full license texts: https://github.com/mpvkit/MPVKit

## Anime4K
Licensed under the MIT License.
Copyright (c) bloc97
Source: https://github.com/bloc97/Anime4K

## Fonts
Licensed under the SIL Open Font License 1.1 (OFL-1.1), bundled via Fontsource (https://fontsource.org):
Inter, Roboto, Nunito, Outfit, Lexend, Atkinson Hyperlegible Next, OpenDyslexic, Literata,
Merriweather, Lora, Source Serif 4, JetBrains Mono.
Full license text: https://openfontlicense.org/open-font-license-official-text/
