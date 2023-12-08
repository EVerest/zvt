# ZVT

This is an implementation of the ECR ZVT Protocoll used in payment terminals
across Europe. It follows the ECR-Interface / ZVT - Protocol specification
defined by the Verband der Terminal Hersteller e.V.. Currently, we follow
Revision 13.11.

The specification can be downloaded
[here](https://www.terminalhersteller.de/downloads.aspx) and the code, as well
as the documentation references sections in this document.

The code also implements extensions defined by
[Feig](https://www.feig-payment.de/), because this is the Terminal that Qwello
uses in production in most of our charging stations.

## Getting started

Start by looking in [`status.rs`](zvt/src/bin/status/main.rs) for a typical way of
interfacing with a terminal. A useful standalone tool is
[`feig_update.rs`](zvt/src/bin/feig_update/main.rs) which we use in production to
update the Firmware of our cVEND plug terminals.
