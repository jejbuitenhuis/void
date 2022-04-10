CREATE TABLE File (
	id BIGINT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
	filename CHAR(128) NOT NULL, -- sha512 hash as hex
	extension VARCHAR(10) NULL,
	mime_type VARCHAR(255) NOT NULL -- https://stackoverflow.com/a/643772/9946744
);
