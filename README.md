## A postgresql extension for snowflake id generating which is implemented with RUST!

### How to use
Download the latest version from the [release](https://github.com/k9982874/pg_snowflake_id/releases/)

Unzip the corresponding release package, like `pg_snowflake_id-17.5-v0.0.1.zip`.

Copy `lib/postgresql/pg_snowflake_id.so` to the lib folder of postgresql. For Debian/Ubuntu like distributions, the folder location is `/usr/lib/postgresql/<VERSION>/lib/`.

Copy files under `share/postgresql/extension` to the system share folder. For Debian/Ubuntu like distributions, the folder location is `/usr/share/postgresql/<VERSION>/extension/`.

Reset the postgresql service. For Debian/Ubuntu like distributions run `systemctl restart postgresql.service`.

### Configuration
The data center id and worker id set to 1 by default, you can change the values by editing the `postgresql.conf` file.

For Debian/Ubuntu like distributions, the folder location is `/etc/postgresql/<VERSION>/main/postgresql.conf`.

Open the `postgresql.conf` with your favorite editor, append the options `pg_snowflake_id.data_center_id` and `pg_snowflake_id.worker_id`.
```
# Example

# others configuration

# Add settings for extensions here
pg_snowflake_id.data_center_id = <DATA_CENTER_ID> # default value is 1
pg_snowflake_id.worker_id = <WORKER_ID> # default value is 1
pg_snowflake_id.epoch = <EPOCH> # default value is 2021-01-01T00:00:00Z
```

**To apply the changes, the postgresql instance must be restarted after each configuration modification。**

### Run extension
Your extension is ready now, let's connect the postgresql instance and try the extension.
```
postgres=# show pg_snowflake_id.data_center_id;
 pg_snowflake_id.data_center_id
--------------------------------
 1 <--- The configured value in postgresql.conf
(1 row)

postgres=# show pg_snowflake_id.worker_id;
 pg_snowflake_id.worker_id
---------------------------
 1 <--- The configured value in postgresql.conf
(1 row)


postgres=# create extension pg_snowflake_id;

Connected to the postgresql instance.

postgres=# create extension pg_snowflake_id;
CREATE EXTENSION

postgres=# select generate_snowflake_id();
 generate_snowflake_id
-----------------------
    576968555229220864
(1 row)
```

### Development
First of all, you need to install the rust development toolchain with, check the details on rust [website](https://www.rust-lang.org/tools/install)

Install `cargo-pgrx` with `cargo install --locked cargo-pgrx@0.14.3`

Install the system dependencies
```
apt install build-essential libreadline-dev zlib1g-dev flex bison libxml2-dev libxslt-dev libssl-dev libxml2-utils xsltproc ccache pkg-config libclang-dev
```

Run `cargo pgrx run` to debug the extension with local postgresql instance.

Run `cargo pgrx package to build the release.

