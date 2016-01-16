# Hatebu to Pocket

## Usage

```
cargo run --release
# => run server port 3000
```

## Setup

### Apache Config

```
<VirtualHost _default_:80>
ServerName <your host name>
ProxyPreserveHost On
ProxyPass /_/hatena_pocket/ http://localhost:3000/_/hatena_pocket/
ProxyPassReverse /_/hatena_pocket/ http://localhost:3000/_/hatena_pocket/
ProxyPass /_/pocket_auth/ http://localhost:3000/_/pocket_auth/
ProxyPassReverse /_/pocket_auth/ http://localhost:3000/_/pocket_auth/
</VirtualHost>
```

### config.toml

```
[hatena]
apikey = "<your hatena bookmark webhook api key>"

[pocket]
redirect_url = "<redirected to after authentication Pocket>/pocket_auth/"
consumer_key = "<pocket application consumer_key>"
mail = "<your pocket mail address>"
```
