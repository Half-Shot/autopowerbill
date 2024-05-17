# autopowerbill
Calculate the cost of power usage from a power monitor, hooked up to Octopus UK Agile energy prices.

This tool will measure the amount of kwh used between monitoring periods, and work out with reasonable
accuracy how much power was used between two points and therefore which prices apply. The more often
power is reported, the better the accuracy.

New prices are automatically fetched every 3 hours.


### Usage

You need to install Rust. And if you don't have it, you have some important life questions to ask yourself.

You can then run `-h` to get the command line arguments.
```sh
cargo run -- -h

Monitors telemetry from a Tasmota Smart Plug and accurate records the costs for each period of use, using Octopus Agile energy prices

Usage: autopowerbill [OPTIONS] --host <HOST> --topic <TOPIC> --product-code <PRODUCT_CODE> --tariff-code <TARIFF_CODE> --database <DATABASE>

Options:
      --host <HOST>                  MQTT broker host name
  -p, --port <PORT>                  MQTT broker port
  -t, --topic <TOPIC>                MQTT topic
      --product-code <PRODUCT_CODE>  Octopus Energy product code
      --tariff-code <TARIFF_CODE>    Octopus Energy tariff code
      --database <DATABASE>          Postgres database connection string
  -h, --help                         Print help
  -V, --version                      Print version
```

For example, you can (after running `cargo install`) do:

```sh
autopowerbill --product-code "AGILE-24-04-03" --tariff-code "E-1R-AGILE-24-04-03-D" --topic  "tele/your-device-topic/SENSOR" --host 'your-broker-address' --database 'postgres://connectionstring'
```

to start recording data. Data is automatically recorded into two CSV files:

- `powerusage.csv` contains the total power usage at a given timestamp.
  - `<date>,<total_energy>`
- `costfile.csv` contains the costs incurred since the last recorded time.
  - `<date>,<usage_over_period>,<total_usage>,<cost_of_usage_over_period>`

