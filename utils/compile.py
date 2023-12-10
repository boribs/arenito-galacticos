import subprocess

def find_port() -> str:
    out = subprocess.run(["arduino-cli", "board", "list"], capture_output=True, text=True)
    ports = []
    for line in out.stdout.split('\n')[1:]:
        if line:
            line = list(map(lambda n: n.strip(), line.split()))
            if 'Unknown' not in line:
                ports.append(line)

    return ports[0][0]

def main():
    port = find_port()
    out = subprocess.run([
        'arduino-cli', 'compile', '-p', port,
        '--fqbn', 'arduino:avr:mega',
        '/Users/boristoteles/Documents/tmr23/arenito/arduino/arenito/arenito.ino'
        ],
        capture_output=True,
        text=True,
    )
    print(out.stdout)
    out = subprocess.run([
        'arduino-cli', 'upload', '-p', port,
        '--fqbn', 'arduino:avr:mega',
        '/Users/boristoteles/Documents/tmr23/arenito/arduino/arenito/arenito.ino'
        ],
        capture_output=True,
        text=True,
    )
    print(out.stdout)

if __name__ == '__main__':
    main()
