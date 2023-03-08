### all params
hash=3365784589&username=MARSHAL&digest=&rid=A76AF7BDD97A2E0F54D6934A8583EFF6916BA1F56E5F2FAC85EE5E447BCA02DD&sid=53219938&realm=INCURSION&realm_digest=17E615D232B76E060567D3327F2D31758F9CCF8ACE0319E1AD49C626F766CA5B

### seaorm cli commands for migration and entity generation with specific dirs
sea-orm-cli.exe migrate generate create_player_table -d .\src\migration\
sea-orm-cli.exe generate entity -u "sqlite://classified.db" -o .\src\entity\src

### varvaytya examples for better logging messages
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

### xml examples
```xml
<data>
    <player hash='193474582' digest='' rid='4D937F46A2FEB79B176AEED6C22D355D0AD57736A338C223F68704AE379CF0D9'>
        <person max_authority_reached="100.000000" authority="100.000000" job_points="999694.000000" faction="0" name="Bobby Mitchell" version="155" alive="1" soldier_group_id="0" soldier_group_name="default" block="8 14" squad_size_setting="-1">
            <order moving="1" target="403.168 0 609.288" class="2" />
            <item slot="0" index="12" amount="1" key="m16a4.weapon" />
            <item slot="1" index="21" amount="1" key="medikit.weapon" />
            <item slot="2" index="60" amount="2" key="hand_grenade.projectile" />
            <item slot="4" index="-1" amount="0" key="" />
            <item slot="5" index="10" amount="1" key="vest2.carry_item" />
            <stash hard_capacity="500">
                <item_group class="0" index="8" key="mossberg.weapon" amount="1" />
                <item_group class="0" index="21" key="medikit.weapon" amount="1" />
                <item_group class="1" index="60" key="hand_grenade.projectile" amount="1" />
                <item_group class="3" index="45" key="camouflage_suit.carry_item" amount="1" />
            </stash>
            <backpack hard_capacity="255">
                <item_group class="0" index="21" key="medikit.weapon" amount="2" />
                <item_group class="0" index="47" key="mortar_resource.weapon" amount="1" />
                <item_group class="0" index="74" key="binoculars.weapon" amount="1" />
                <item_group class="3" index="10" key="vest2.carry_item" amount="1" />
            </backpack>
        </person>
        <profile game_version="155" username="WTF" digest="" sid="53219938" rid="4D937F46A2FEB79B176AEED6C22D355D0AD57736A338C223F68704AE379CF0D9" squad_tag="" color="1 1 1 0">
            <stats kills="0" deaths="0" time_played="110.000000" player_kills="0" teamkills="0" longest_kill_streak="0" targets_destroyed="0" vehicles_destroyed="0" soldiers_healed="0"
                   times_got_healed="0" distance_moved="75.942490" shots_fired="0" throwables_thrown="0" rank_progression="0.000000">
                <monitor name="kill combo">
                    <entry combo="3" count="4" />
                    <entry combo="4" count="0" />
                    <entry combo="5" count="1" />
                </monitor>
                <monitor name="death streak" longest_death_streak="0" />
                <monitor /><monitor /><monitor />
                <monitor name="destroyer" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="saboteur" level="0">
                    <criteria count="0" />
                    <criteria count="0" />
                </monitor>
                <monitor name="tracker" level="0">
                    <criteria count="0" />
                    <criteria count="0" />
                </monitor>
                <monitor name="killer" level="0">
                    <criteria count="53" />
                </monitor>
                <monitor name="stealth" level="0">
                    <criteria count="0" />
                    <criteria count="0" />
                    <criteria count="0" />
                </monitor>
                <monitor name="journal_deaths_at_start_trigger" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="difficulty_change_trigger" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_1st_map_victory" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_deaths" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_miniboss_kill" level="1">
                    <criteria count="1" />
                </monitor>
                <monitor name="steam_green_campaign_complete" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_gray_campaign_complete" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_brown_campaign_complete" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_darkcat_destroyed" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_roadkills" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_medic" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_enemy_weapon_delivered" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_prison_destroyed" level="0">
                    <criteria count="0" />
                </monitor>
                <monitor name="steam_blast_kills" level="0">
                    <criteria count="7" />
                </monitor>
                <monitor name="steam_shots_fired" level="0">
                    <criteria count="975" />
                </monitor>
                <monitor name="steam_antiair_destroyed" level="0">
                    <criteria count="0" />
                </monitor>
            </stats>
        </profile>
    </player>
</data>
```

```xml
<data ok="1">
    <profile game_version="155" username="WTF" sid="53219938" rid="4D937F46A2FEB79B176AEED6C22D355D0AD57736A338C223F68704AE379CF0D9" squad_tag="">
        <stats kills="0" deaths="0" time_played="110" player_kills="0" teamkills="0" longest_kill_streak="0" targets_destroyed="0" vehicles_destroyed="0"
               soldiers_healed="0" distance_moved="75.94249" shots_fired="0" throwables_thrown="0" rank_progression="0"/>
    </profile>
    <person max_authority_reached="100" authority="100" job_points="999694" faction="0" name="Bobby Mitchell" soldier_group_id="0" soldier_group_name="default" squad_size_setting="-1">
        <item slot="0" index="12" amount="1" key="m16a4.weapon"/>
        <item slot="1" index="21" amount="1" key="medikit.weapon"/>
        <item slot="2" index="60" amount="2" key="hand_grenade.projectile"/>
        <item slot="4" index="-1" amount="0" key=""/>
        <item slot="5" index="10" amount="1" key="vest2.carry_item"/>
        <backpack>
            <item_group class="0" index="21" key="medikit.weapon" amount="2"/>
            <item_group class="0" index="47" key="mortar_resource.weapon" amount="1"/>
            <item_group class="0" index="74" key="binoculars.weapon" amount="1"/>
            <item_group class="3" index="10" key="vest2.carry_item" amount="1"/>
        </backpack>
        <stash>
            <item_group class="0" index="8" key="mossberg.weapon" amount="1"/>
            <item_group class="0" index="21" key="medikit.weapon" amount="1"/>
            <item_group class="1" index="60" key="hand_grenade.projectile" amount="1"/>
            <item_group class="3" index="45" key="camouflage_suit.carry_item" amount="1"/>
        </stash>
    </person>
</data>
```