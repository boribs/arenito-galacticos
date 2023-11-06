import subprocess, sys

def find_port() -> str:
    out = subprocess.run(['arduino-cli', 'board', 'list'], capture_output=True, text=True)
    ports = []
    for line in out.stdout.split('\n')[1:]:
        if line:
            line = list(map(lambda n: n.strip(), line.split()))
            if 'Unknown' not in line:
                ports.append(line)

    return ports[0][0]

def main():
    try:
        filepath = sys.argv[1]
    except:
        print('Must provide file to compile and upload!')
        exit(1)

    port = find_port()

    out = subprocess.run([
        'arduino-cli', 'compile', '-p', port,
        '--fqbn', 'arduino:avr:mega',
        filepath
        ],
        capture_output=True,
        text=True,
    )
    print(out.stdout)
    out = subprocess.run([
        'arduino-cli', 'upload', '-p', port,
        '--fqbn', 'arduino:avr:mega',
        filepath
        ],
        capture_output=True,
        text=True,
    )
    print(out.stdout)

if __name__ == '__main__':
    main()
