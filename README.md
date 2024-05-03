# autopowerbill
Calculate the cost of power usage from a power monitor, hooked up to Octopus UK energy prices.


### Usage

You need to install Rust. And if you don't have it, you have some important life questions to ask yourself.

Fill out two files in the `data/` directory:

- `avgpowercost.csv` containing the average cost of power for a given date. It is in the format of <dd-mm-yy>,<gbp-pence>.
- `powerusage.csv`containing the power usage for a given date. It is in the format of <dd-mm-yy>,<kwh>.

```sh
# cargo run -- <optional_base_charge_in_pence>
cargo run -- 500
Calculated cost: £9.16
```
