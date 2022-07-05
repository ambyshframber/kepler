# kepler

a simple gemini server

## a brief note about gemini

gemini is like the web, but smaller. it's designed to be a much smaller specification and therefore much easier to implement and maintain.

for more info, check out [this page](https://gemini.circumlunar.space/)

## how to use kepler

kepler uses INI for config. to start kepler, run `kepler path/to/config/file`. all paths in the config are relative to the folder containing the config file. an example setup is included in this repository. values MUST NOT be enclosed with quotes.

list of keys:

- `hostname`: should be set to the hostname of the server. this is used to validate request URIs. defaults to `localhost`
- `private_key_file`: the path to the private key file. the file should be PEM encoded. defaults to `key.pem`
- `cert_chain_file`: the path to the TLS certificate. again, the file should be PEM encoded. defaults to `cert.pem`
- `content_root`: the folder to use when serving a static file. defaults to `content`
- `index`: the file to serve when a request path terminates at a folder. defaults to `index.gmi`
- `port`: the port to use. this should never really need changing. defaults to `1965`
- `redirects_file`: the path to the redirects table file. if not present, no redirects will be loaded.
- `redirects_ttl`: the amount of time to wait before refreshing the redirects table from the disk. defaults to `1800`, equal to 30 minutes

### redirects

kepler supports redirections, and can dynamically update redirects without needing to restart. redirects are specified in a separate INI file, which is pointed to by the `redirects_file` key. this file is structured a little oddly because gemini redirects can be permanent or temporary. an example redirect might be
```
[test_redirect]
destination = gemini://gemini.circumlunar.space
permanent = true
```

any request for that path will be redirected to `destination`, with the permanence of the redirect being determined by `permanent`. if permanent is not present, the redirect will be assumed to be permanent. paths do not have a starting or ending slash.

the redirect table will be reloaded from disk on the first incoming request after N seconds have passed since the last reload, where N is the value of `redirects_ttl`.
