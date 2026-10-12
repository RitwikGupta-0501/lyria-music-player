import re
with open('src-tauri/src/db/queries.rs', 'r') as f:
    content = f.read()

# Replace SELECT queries to include genre
content = content.replace("SELECT id, title, artist, album_id, track_number, file_path FROM tracks", "SELECT id, title, artist, album_id, genre, track_number, file_path FROM tracks")
content = content.replace("SELECT t.id, t.title, t.artist, t.album_id, t.track_number, t.file_path FROM tracks", "SELECT t.id, t.title, t.artist, t.album_id, t.genre, t.track_number, t.file_path FROM tracks")

# Now update the struct initializations
content = re.sub(
    r"album_id: row.get\((\d+)\)\?,\s+track_number: row.get\((\d+)\)\?,",
    lambda m: f"album_id: row.get({m.group(1)})?,\n            genre: row.get({int(m.group(1))+1})?,\n            track_number: row.get({int(m.group(2))+1})?,",
    content
)

# And file_path gets incremented too
content = re.sub(
    r"file_path: row.get\((\d+)\)\?,",
    lambda m: f"file_path: row.get({int(m.group(1))+1})?,",
    content
)

with open('src-tauri/src/db/queries.rs', 'w') as f:
    f.write(content)
