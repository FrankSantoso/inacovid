# INACOVID

Gets Covid-19 data from Indonesian Government source.

### Initial Setup:

* First migrate the schema to your postgresql database

    ```psql -U <postgres_user_name> -h <postgres_host> -d <database_name> < schema.sql```

* Change the provided ```config.json``` to reflect your database config

* Build the binary using ```make & make INSTALLDIR=/your/path/to/bin/here install```


### CLI Usage:

```
inacovid 0.1
Alexander Adhyatma <alex@asiatech.dev>
Gets Indonesian Covid19 data from gov't source, save it to postgres and json dir

USAGE:
    inacovid --config <config>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Config file containing database dsn and json output dir

```

Sets a crontab if you wishes to run it periodically.
