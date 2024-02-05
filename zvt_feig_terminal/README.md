# ZVT Feig Terminal

The crate implements the application logic for using a Feig terminal in production.

We assume that you interact with the Feig terminal over TCP/IP.

The high level interface for interacting with the Feig terminal is implemented
in [src/feig.rs](src/feig.rs). It provides good defaults for reading cards and
allows you to begin, commit or cancel transactions.
