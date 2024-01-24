CREATE TABLE discord_guild (
	id NUMERIC PRIMARY KEY
);

CREATE TABLE discord_channel (
	id NUMERIC PRIMARY KEY,
	guild_id NUMERIC NOT NULL,

	FOREIGN KEY (guild_id) REFERENCES discord_guild(id)
);

CREATE TABLE discord_user (
	id NUMERIC PRIMARY KEY
);

CREATE TABLE booru_setting (
	id SERIAL PRIMARY KEY,
	
	guild_id NUMERIC,
	user_id NUMERIC,
	channel_id NUMERIC,

	FOREIGN KEY (guild_id) REFERENCES discord_guild(id) ON DELETE CASCADE,
	FOREIGN KEY (channel_id) REFERENCES discord_channel(id) ON DELETE CASCADE,
	FOREIGN KEY (user_id) REFERENCES discord_user(id) ON DELETE CASCADE
);

CREATE TABLE booru_blacklisted_tag (
	id serial primary key,

	booru_setting_id int not null,
	blacklisted varchar(255) not null,

	FOREIGN KEY (booru_setting_id) REFERENCES booru_setting(id) ON DELETE CASCADE,
	UNIQUE (blacklisted, booru_setting_id)
);
