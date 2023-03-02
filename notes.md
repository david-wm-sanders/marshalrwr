2023-01-31T20:06:50.935900Z
DEBUG
request{method=GET 
    uri=/get_profile.php?
    hash=3365784589&
    username=MARSHAL&
    digest=&
    rid=A76AF7BDD97A2E0F54D6934A8583EFF6916BA1F56E5F2FAC85EE5E447BCA02DD&
    sid=53219938&
    realm=INCURSION
    &realm_digest=17E615D232B76E060567D3327F2D31758F9CCF8ACE0319E1AD49C626F766CA5B
version=HTTP/1.1}: 
tower_http::trace::on_request: started processing request

# all params
hash=3365784589&username=MARSHAL&digest=&rid=A76AF7BDD97A2E0F54D6934A8583EFF6916BA1F56E5F2FAC85EE5E447BCA02DD&sid=53219938&realm=INCURSION&realm_digest=17E615D232B76E060567D3327F2D31758F9CCF8ACE0319E1AD49C626F766CA5B

# seaorm cli commands for migration and entity generation with specific dirs
sea-orm-cli.exe migrate generate create_player_table -d .\src\migration\
sea-orm-cli.exe generate entity -u "sqlite://classified.db" -o .\src\entity\src

# varvaytya examples for better logging messages
2023-02-22 03:10:07.609 | INFO     |�  [get] Validating request args...
2023-02-22 03:10:07.610 | DEBUG    |�  [get] Locating realm 'INCURSION' and checking digest...
2023-02-22 03:10:07.674 | INFO     |�  [get] Located realm 'INCURSION' (1) [world: valhalla]
2023-02-22 03:10:07.676 | DEBUG    |�  [get] Identifying 'MARSHAL' and checking papers (rid/sid)...
2023-02-22 03:10:07.679 | INFO     |�  [get] Player 'MARSHAL' [3365784589] not found in db, enlisting...
2023-02-22 03:10:07.680 | INFO     |�  [get] Creating new account (1, 3365784589) for 'MARSHAL' in 'INCURSION'...
2023-02-22 03:10:07.697 | DEBUG    |�  [get] Constructing xml for 'MARSHAL' in 'INCURSION' (1, 3365784589)..
...
2023-02-22 03:11:40.534 | INFO     |�  [get] Processing request from '127.0.0.1'...
2023-02-22 03:11:40.537 | DEBUG    |�  [get] request args: hash=3365784589, username=MARSHAL, sid=53219938, realm=INCURSION, realm_digest=17E615D232B76E060567D3327F2D31758F9CCF8ACE0319E1AD49C626F766CA5B
2023-02-22 03:11:40.538 | INFO     |�  [get] Validating request args...
2023-02-22 03:11:40.539 | DEBUG    |�  [get] Locating realm 'INCURSION' and checking digest...
2023-02-22 03:11:40.542 | INFO     |�  [get] Located realm 'INCURSION' (1) [world: valhalla]
2023-02-22 03:11:40.543 | DEBUG    |�  [get] Identifying 'MARSHAL' and checking papers (rid/sid)...
2023-02-22 03:11:40.544 | INFO     |�  [get] Verified player 'MARSHAL' (3365784589) [sid: None]
2023-02-22 03:11:40.545 | DEBUG    |�  [get] Finding account (1, 3365784589)...
2023-02-22 03:11:40.571 | DEBUG    |�  [get] Constructing xml for 'MARSHAL' in 'INCURSION' (1, 3365784589)...


DEBUG request{method=POST uri=/set_profile.php?realm=INCURSION&realm_digest=17E615D232B76E060567D3327F2D31758F9CCF8ACE0319E1AD49C626F766CA5B}