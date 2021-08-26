# Mac Hori USB Controller
I only had access to a mac and a hori fighting commander for switch, and macs can't seem to read most USB controllers natively.
Maps inputs to Keyboard buttons - which buttons it maps to are not customizable at the moment.

## Running
`make` or `make debug=1`
`RUST_LOG={log_level} target/{target}/mac-usb-controller`

## Supports
* Hori Fighting commander (switch)
* Possibly other Hori switch devices?
