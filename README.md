## ardina: multi-channel news delivery system

Ardina takes your RSS feed and delivers them to subscribers via email (and
maybe twitter one day).


## Running ardina

Ardina fetches the feed, checks for new items (storing the GUID of seen items
in the database), and emails any unseen items via email to the subscriber list.

If it fails sending

It fetches the feed only once, so you'll want to set it up in a cron job or
something depending on the feed update frequency.


## Configuration

Ardina assumes the existence of a `ardina.toml` configuration file located in
the current working directory. Here's a complete example of a configuration
file, where all values are required:

```toml
[feed]
url = "https://example.org/index.xml"

[database]
url = "production.sqlite3"

[email]
relay = "mx.example.org"
username = "newsletter@example.org"
password = "mypassword"
from = "Example Newsletter <newsletter@example.org>"
subject_prefix = "[Example Newsletter]"
subscribers = [
  "Hugo Peixoto <wiki@example.org>",
]
```


## Missing features

- error handling: unwraps everywhere;
- retries: if delivering an item to a subscriber fails, it won't send it to the
  following subscribers, and it will mark the item as seen;
- optional `email.reply_to` setting
- maybe track bounces / delivery failures?
- text multipart instead of only sending html
- header / footer information with unsubscribe links and all of that
- twitter integration
