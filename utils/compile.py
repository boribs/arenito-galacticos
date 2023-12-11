import subprocess, sys

def find_arduino() -> [str, str]:
    out = subprocess.run(['arduino-cli', 'board', 'list'], capture_output=True, text=True)
    ports = []
    for line in out.stdout.split('\n')[1:]:
        if line:
            line = list(map(lambda n: n.strip(), line.split()))
            if 'Unknown' not in line:
                ports.append(line)

    if len(ports) == 0:
        raise Exception('No Arduinos detected.')
    elif len(ports) > 1:
        raise Exception('More than one Arduino connected!')

    return ports[0][0], ports[0][7]

def main():
    try:
        filepath = sys.argv[1]
    except:
        print('Must provide file to compile and upload!')
        exit(1)

    port, model = find_arduino()

    out = subprocess.run([
        'arduino-cli', 'compile', '-p', port,
        '--fqbn', model,
        filepath
        ],
        capture_output=True,
        text=True,
    )
    print(out.stdout if out.stdout else out.stderr)
    out = subprocess.run([
        'arduino-cli', 'upload', '-p', port,
        '--fqbn', model,
        filepath
        ],
        capture_output=True,
        text=True,
    )
    print(out.stdout if out.stdout else out.stderr)

if __name__ == '__main__':
    main()
