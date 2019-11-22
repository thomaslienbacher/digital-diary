# Digital Diary

`didi` is CLI program that allows you to document your daily life.

### How it works

A diary has entries which are stored in a SQLite3 database.
*Warning __no__ encryption is used!* Each entry consists of a title,
keywords and content. Inside the database metadata such as
id, hash, date and time of creation, hidden status are also stored.

### How to use it

`$ didi create` creates the database. It uses the environment
variable `DIDI_URL` to determine the location, if nothing is
specified the database will be created in the user directory.

`$ didi add` add an entry to the database. The content can be
multiline in order to confirm the content `ENTER` must be pressed
twice. Keywords are case-insensitive and seperated using space.

`$ didi list` lists all entries. The entries id and hash can be
displayed using flags. 

`$ didi search <to-search>...` searches for an entry based on the
title and keywords.

`$ didi hide <id>...` hides an entry. This means it won't be
displayed unless a flag is used.

`$ didi unhide <id>...` unhides an entry. 

`$ didi help <subcommand>` get more help on a specify command.

For full help information use `$ didi -h`.
