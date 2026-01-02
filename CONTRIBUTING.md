
# Contributing to SkyPulseDB

Thank you for your interest in contributing! SkyPulseDB is an open-source, Rust-based time-series database for weather and meteorological data. We welcome bug reports, feature requests, documentation improvements, and code contributions.

## Getting Started

1. **Fork the repository** and clone your fork locally.
2. **Install Rust** (latest stable) from [rustup.rs](https://rustup.rs).
3. **Build and test**:
	```bash
	cargo build
	cargo test
	```
4. **Run the server**:
	```bash
	cargo run -- --data-dir ./data
	```

## Code Style

- Use `rustfmt` for formatting and `clippy` for linting.
- Write clear, concise comments and documentation.
- Prefer small, focused commits.

## Submitting Changes

1. Open a pull request (PR) against the `main` branch.
2. Ensure your PR passes all tests (`cargo test`).
3. Add or update documentation as needed.
4. Reference related issues in your PR description.

## Bug Reports & Feature Requests

- Use [GitHub Issues](https://github.com/CasperHK/SkyPulseDB/issues) for bugs and feature requests.
- Include steps to reproduce, expected behavior, and environment details.

## Code of Conduct

Please be respectful and inclusive. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for details.

## License

By contributing, you agree your work will be licensed under the Apache 2.0 License.

---

Happy coding! 🚀
