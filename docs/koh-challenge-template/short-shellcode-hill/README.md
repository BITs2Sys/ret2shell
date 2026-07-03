Submit x86_64 Linux shellcode to the shared hill. A valid payload must write exactly `KOH\n` to stdout. Each round lasts 60 seconds. For every completed round, teams are ranked by the shortest valid shellcode length; ties are broken by earlier submission time.

Use the KoH tab to copy your team identifier, then submit:

```bash
python3 static/submit_shellcode.py http://TARGET:8080 koh_xxxxxxxxxxxxxxxx 4831c0b00148c7c70100000048bb4b4f480a00000000534889e6b2040f05b03c4831ff0f05
```

The first five teams of each round receive 100%, 80%, 60%, 40%, and 20% of the per-round score.
