# swapAnniversary
To make the API accesible change 127.0.0.01 in main.rs to 0.0.0.0
Before running the bot create a directory in the swap directory (not in /src) called "databases", and create a sqlite database called "swapAnniversary.db" and execute the create tables command found in tables.sql

Env variables:
TOKEN: token for the discord bot
HOST_ROLE: host role, allowed to use special commands
GUILD_ID: the server ID of the server that the bot will be running on
PHASE: the number of the phase (set to 0 to make sure it starts with no commands allowed)