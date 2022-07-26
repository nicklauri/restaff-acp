# restaff-acp — Auto claim points on Restaff newsfeed page.

Build instructions:
  1. Install Rust via [rustup](https://rustup.rs/) by downloading from the page or running `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` if you're on *nix-like environment.
  2. Get source code Restaff-ACP: `git clone https://github.com/nicklauri/restaff-acp`
  3. Build binary:
   ```
   $ cd restaff-acp
   $ cargo build --release
   ```
  4. Typing `cargo run --release -- -h` to get help. The executable file is at `./target/release`
  5. Optional: create a crontab (GNU/Linux) or task in task scheduler (Windows) to run daily.

Suggestions:
  - Create a new task to run this program daily.
  - Password file can store only a single hashed base64 or with username with format: `username:password`. Since your password is encrypted with base64, **USE AT YOUR OWN RISK**
  - Note that password has stored in base64 algorithm, so it's easy to decode. Be careful.
  - If only `-u/--username` option specified, a prompt to input password with be displayed.

Usage:
```
$ restaff-acp.exe -h
restaff-claim-points 0.1.0
Claim point from Restaff page

USAGE:
    restaff-acp.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --api-server <api-server>           [default: https://api.gigging.tech]
    -c, --claim-type <claim-type>           [default: 3]
    -p, --password <password>
    -f, --password-file <password-file>
    -u, --username <username>
```
