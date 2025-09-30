# osrm-interface

Rust interface for [OSRM (Open Source Routing Machine)](http://project-osrm.org/). Includes low-level native bindings to osrm-backend and a wrapper for the HTTP request server. In addition, provides a mocking mode for simplified development. The native bindings provides an idiomatic and type-safe interface to access core OSRM functionalities (`route`, `table`, `trip`) from Rust.

Forked from [osrm-binding](https://github.com/mathias-vandaele/osrm-binding) which provided the initial FFI interface for this crate and also supports building of OSRM itself, unlike this fork.


## Building native feature

Native requires a built version of [osrm-backend](https://github.com/Project-OSRM/osrm-backend) with it's installed headers added to the include path. A fully installed osrm-backend will place all required headers on the include path by default. To call into osrm-backend natively, there is a `.cpp` wrapper which provides `C` bindings to the required osrm services. To build this wrapper requires a `C++` compiler which supports `std=c++17`. I have successfully built it with `g++-12` and newer versions should work also. Clang should also work but is untested.

Cargo uses your default `c++` compiler by default, to override this, simply 

```bash
export CXX=g++-12 # whichever compiler you have
```

In addition, to run natively requires appropriately extracted osrm-map data. As described on [osrm-backend](https://github.com/Project-OSRM/osrm-backend), map data can be obtained from [Geofabrik](https://download.geofabrik.de/) and then needs to be extracted with `osrm-extract` before being partitioned (`osrm-partition`) and customized (`osrm-customize`) for MLD or contracted (`osrm-contract`) for CH.

The path to the map data needs to be specified in the `.env` file to run the tests which attempt to route in Germany.


## 📖 License

This project is licensed under the MIT License.

---

Made with ❤️ in Rust.