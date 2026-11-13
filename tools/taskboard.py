import argparse
import os
import sqlite3
import sys
import time

DB_PATH = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "taskboard.db")


def connect():
    conn = sqlite3.connect(DB_PATH, timeout=30)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    return conn


def init():
    conn = connect()
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            owner TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'todo',
            notes TEXT DEFAULT '',
            created_at REAL NOT NULL,
            updated_at REAL NOT NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            sender TEXT NOT NULL,
            text TEXT NOT NULL,
            created_at REAL NOT NULL,
            FOREIGN KEY(task_id) REFERENCES tasks(id)
        );
        """
    )
    conn.commit()
    conn.close()


def now():
    return time.time()


def cmd_add(args):
    init()
    conn = connect()
    cur = conn.execute(
        "INSERT INTO tasks (title, owner, status, notes, created_at, updated_at) VALUES (?,?,?,?,?,?)",
        (args.title, args.owner, "todo", args.notes or "", now(), now()),
    )
    conn.commit()
    print(cur.lastrowid)
    conn.close()


def cmd_claim(args):
    init()
    conn = connect()
    conn.execute(
        "UPDATE tasks SET owner=?, status='running', updated_at=? WHERE id=?",
        (args.owner, now(), args.id),
    )
    conn.commit()
    conn.close()
    print(f"claimed {args.id} by {args.owner}")


def cmd_update(args):
    init()
    conn = connect()
    conn.execute(
        "UPDATE tasks SET status=?, notes=COALESCE(?,notes), updated_at=? WHERE id=?",
        (args.status, args.note, now(), args.id),
    )
    conn.commit()
    conn.close()
    print(f"updated {args.id} -> {args.status}")


def cmd_message(args):
    init()
    conn = connect()
    conn.execute(
        "INSERT INTO messages (task_id, sender, text, created_at) VALUES (?,?,?,?)",
        (args.id, args.sender, args.text, now()),
    )
    conn.commit()
    conn.close()
    print(f"message added to {args.id}")


def cmd_list(args):
    init()
    conn = connect()
    rows = conn.execute(
        "SELECT id, title, owner, status, notes, updated_at FROM tasks ORDER BY id"
    ).fetchall()
    for r in rows:
        print(f"[{r[0]}] {r[3]:8} owner={r[2]:12} title={r[1]}")
        if r[4]:
            print(f"    notes: {r[4]}")
        msgs = conn.execute(
            "SELECT sender, text FROM messages WHERE task_id=? ORDER BY id", (r[0],)
        ).fetchall()
        for m in msgs:
            print(f"    msg {m[0]}: {m[1]}")
    conn.close()


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd", required=True)

    a = sub.add_parser("add")
    a.add_argument("--title", required=True)
    a.add_argument("--owner", default="orchestrator")
    a.add_argument("--notes", default="")
    a.set_defaults(func=cmd_add)

    c = sub.add_parser("claim")
    c.add_argument("--id", type=int, required=True)
    c.add_argument("--owner", required=True)
    c.set_defaults(func=cmd_claim)

    u = sub.add_parser("update")
    u.add_argument("--id", type=int, required=True)
    u.add_argument("--status", required=True)
    u.add_argument("--note", default=None)
    u.set_defaults(func=cmd_update)

    m = sub.add_parser("message")
    m.add_argument("--id", type=int, required=True)
    m.add_argument("--from", dest="sender", required=True)
    m.add_argument("--text", required=True)
    m.set_defaults(func=cmd_message)

    l = sub.add_parser("list")
    l.set_defaults(func=cmd_list)

    args = p.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
