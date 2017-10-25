# Getting Started With rust2email

rust2email is a mostly-compatible rewrite of rss2email.

# Installing rust2email

Currently, the only way is to download this repository and use cargo

```bash
  $ cargo build --release
```

# Using rust2email

Unlike rss2email, there is no need to explicitly build a database,
it will be created if not present and saved at each execution

There is two files used by rust2email: configuration file and database file,
both use the XDG convention and can be specified on the command line.

## Config file

The config file use the [TOML](https://github.com/toml-lang/toml) syntax

```toml
# ~/.config/rust2email/rust2email.toml

# Mandatory values:
to = "postmaster@invalid"
email_backend = "sendmail" # current possibilities are: sendmail, file

# Other values:

#verbose = false
# default config send html mail
#text = false
#text_wrap = 80
#from_address = "user@rust2email.invalid"
#from_display_name = "<feed_name>"
#subject = "<entry_name>"
#body = "<p>URL: <entry_url></p>\r\n<entry_body>"

#[mail_file]
#path = "test"

#[mail_sendmail]
#path = "/usr/sbin/sendmail"

```

Subscribe to some feeds

```bash
  $ rust2email add feed_name feed_url
```

or

```bash
  $ rust2email opmlimport <opmlfile>
```

When you run rust2email, it emails you about every story it hasn't seen
before. But the first time you run it, that will be every story. To
avoid this, you can ask rust2email not to send you any stories the
first time you run it

```bash
  $ rust2email run --n
```

Then later, you can ask it to email you new stories

```bash
  $ rust2email run
```
