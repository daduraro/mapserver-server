CREATE TABLE points (
    id INTEGER PRIMARY KEY NOT NULL,
    mapid INTEGER NOT NULL,
    coordx REAL NOT NULL,
    coordy REAL NOT NULL,
    title TEXT,
    body TEXT,
    FOREIGN KEY(mapid) REFERENCES maps(id)
); 
