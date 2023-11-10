### seaorm cli commands for migration and entity generation with specific dirs
sea-orm-cli.exe migrate generate create_player_table -d .\src\migration\
sea-orm-cli.exe generate entity -u "sqlite://classified.db" -o .\src\entity\src
