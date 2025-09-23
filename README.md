# osrm-interface

Rust interface for [OSRM (Open Source Routing Machine)](http://project-osrm.org/). Includes low-level native bindings to osrm-backend and a wrapper for the HTTP request server. In addition, provides a mocking mode for simplified development. The native bindings provides an idiomatic and type-safe interface to access core OSRM functionalities (`route`, `table`, `trip`) from Rust.

Forked from [osrm-binding](https://github.com/mathias-vandaele/osrm-binding) which provided the initial FFI interface for this crate and also supports building of OSRM itself, unlike this fork.

## 📖 License

This project is licensed under the MIT License.

---

Made with ❤️ in Rust.