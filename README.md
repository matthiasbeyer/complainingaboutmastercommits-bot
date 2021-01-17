# complainingaboutmastercommits-bot

This is a bot that posts to mastodon how many commits in the last N hours on
branch B were direct commits and how many were merges.

This can be used to complain about people pushing to master directly and thus
possibly breaking CI.

The message to post is configurable, as is the path to the repository, the name
of the master branch and the name of the "upstream" remote.


## Notes on Code of Conduct

This bot can and should NOT be used to discredit single users. Patches adding
such features won't be merged.
Pushes to master branch and breaking CI is not a problem of a single user, but
of as a community as a whole.

**DON'T USE THIS BOT TO BE RUDE!**

See the `status_template` example in the `config.example.toml` file how you
could write your status message without discrediting a community.

Write _facts_, not _opinions_!


## Usage

See the `config.example.toml` for how to configure this bot.
Run it somewhere and maybe trigger it through systemd timers.
This is not a always-running-process bot, but a one-shot program.


## Contributing

Patches are welcome. Send them to my mail address.
Make sure to `--signoff` your commits.


## License

AGPL-3.0-only
(c) Matthias Beyer

