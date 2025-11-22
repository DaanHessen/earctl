<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project_license][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<br />
<div align="center">
  <!-- <a href="https://github.com/DaanHessen/earctl">
    <img src="docs/ear-web/res/icons/256x256.png" alt="Logo" width="80" height="80"> -->
  </a>

<h3 align="center">Nothing Ear API/CLI</h3>

  <p align="center">
    A Rust API/CLI that allows every device on your network to control your Nothing earbuds.
    <br />
    <a href="https://daanhessen.github.io/earctl/"><strong>View full docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/DaanHessen/earctl/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/DaanHessen/earctl/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

## About The Project

<!-- [![Product Name Screen Shot][product-screenshot]](https://github.com/DaanHessen/earctl) -->

earctl is a Rust rewrite of the backend logic used by the community ear-web tool. It exposes the same Bluetooth Serial Port Profile commands through a REST API and a small CLI. That means you can automate ANC, EQ, latency mode, gestures, and the rest of the Nothing/CMF settings without launching a browser.

### Documentation

- Full docs: https://daanhessen.github.io/earctl/
- Static copy in this repo: `docs/index.html`

## Getting Started

Follow these steps to run the API locally. See the docs for deployment, API, and CLI details.

### Prerequisites

* Rust toolchain (1.75+ recommended)
* `bluez`, `bluez-utils` (for `bluetoothctl`)
* `bluez-deprecated-tools` (provides `sdptool` for RFCOMM channel discovery)
* Your earbuds paired in the desktop Bluetooth UI

### Quick install

```sh
git clone https://github.com/DaanHessen/earctl.git
cd earctl
cargo build --release
./target/release/earctl server --addr 0.0.0.0:8787
```

On Arch, install from the AUR with `yay -S earctl` (includes a user service).

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Usage

1. Ensure the server is running (`earctl server --addr 0.0.0.0:8787`) and your earbuds are connected via the OS Bluetooth menu.
2. Auto-connect from any terminal:
   ```sh
   earctl auto-connect --name "Nothing Ear"
   ```
   The server queries BlueZ for connected devices, discovers the Nothing RFCOMM channel via `sdptool`, and opens the session. Supply `--channel <number>` if you need to override the detected value (defaults to channel 1 if detection fails).
3. Toggle ANC, EQ, latency, gestures, LED color, or trigger Find My Buds either via the REST endpoints or the CLI.

_For more information, refer to the [documentation](https://daanhessen.github.io/earctl/)_

## Contributing

Contributions are welcome: open an issue if you found a bug or send a pull request with the fix.

Top contributors:

<a href="https://github.com/DaanHessen/earctl/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=DaanHessen/earctl" alt="contrib.rocks image" />
</a>



## License

Distributed under the GNU Affero GPL-3.0 license. See `LICENSE` for details.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Contact

Daan Hessen – daanh2002@gmail.com

Project Link: [https://github.com/DaanHessen/earctl](https://github.com/DaanHessen/earctl)



## Acknowledgments

* [RapidZapper, Bendix, Lisra-git, MemerGamer and the rest of the ear-web team for the groundwork.](https://github.com/radiance-project/ear-web)
* The BlueZ maintainers.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

[contributors-shield]: https://img.shields.io/github/contributors/DaanHessen/earctl.svg?style=for-the-badge
[contributors-url]: https://github.com/DaanHessen/earctl/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/DaanHessen/earctl.svg?style=for-the-badge
[forks-url]: https://github.com/DaanHessen/earctl/network/members
[stars-shield]: https://img.shields.io/github/stars/DaanHessen/earctl.svg?style=for-the-badge
[stars-url]: https://github.com/DaanHessen/earctl/stargazers
[issues-shield]: https://img.shields.io/github/issues/DaanHessen/earctl.svg?style=for-the-badge
[issues-url]: https://github.com/DaanHessen/earctl/issues
[license-shield]: https://img.shields.io/github/license/DaanHessen/earctl.svg?style=for-the-badge
[license-url]: https://github.com/DaanHessen/earctl/blob/master/LICENSE
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://www.linkedin.com/in/daan-hessen-552789236/
[product-screenshot]: docs/ear-web/res/MainControl/hero.png
[Rust-img]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
[Tokio-img]: https://img.shields.io/badge/Tokio-000000?style=for-the-badge&logo=rust&logoColor=white
[Tokio-url]: https://tokio.rs/
[Axum-img]: https://img.shields.io/badge/Axum-1E88E5?style=for-the-badge&logo=rust&logoColor=white
[Axum-url]: https://github.com/tokio-rs/axum
[BlueZ-img]: https://img.shields.io/badge/BlueZ-0A5CAB?style=for-the-badge&logo=bluetooth&logoColor=white
[BlueZ-url]: http://www.bluez.org/
