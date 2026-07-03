Submit a minion strategy program of at most 16 KiB. At the end of each round, the hill runs a deterministic arena between all submitted strategies and ranks teams by match points, then kills, then remaining base health.

This is a clean-room Ret2Shell sample inspired by SekaiCTF's `game_minions-in-16k` format. The original challenge used a custom game and rating pipeline; this sample keeps the same operational shape for local KoH testing: a shared game server, 16 KiB submissions, completed-round rankings, and platform-side score awards.

Use the KoH tab to copy your team identifier, then submit:

```bash
python3 static/submit_strategy.py http://TARGET:8080 koh_xxxxxxxxxxxxxxxx static/sample_strategy.txt
```

Strategy format is line-oriented. Unknown lines are ignored. Example:

```text
attack=70
gather=45
defend=25
scout=10
```

The first five teams of each round receive 100%, 80%, 60%, 40%, and 20% of the per-round score.
