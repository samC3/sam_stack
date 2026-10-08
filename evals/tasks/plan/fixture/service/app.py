import sqlite3
import time

DB = sqlite3.connect("profiles.db", check_same_thread=False)


def get_profile(user_id):
    # Slow: joins three tables and computes follower counts on every request.
    time.sleep(0.4)
    row = DB.execute(
        "SELECT u.id, u.name, COUNT(f.follower_id) FROM users u "
        "LEFT JOIN followers f ON f.user_id = u.id WHERE u.id = ? GROUP BY u.id",
        (user_id,),
    ).fetchone()
    return {"id": row[0], "name": row[1], "followers": row[2]}


def update_profile(user_id, name):
    DB.execute("UPDATE users SET name = ? WHERE id = ?", (name, user_id))
    DB.commit()
