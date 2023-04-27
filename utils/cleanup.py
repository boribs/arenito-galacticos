# A little program to delete those images that dont have
# their respective .xml

# python cleanup.py folder1 folder2 ...

import sys, os

for folder in sys.argv[1:]:
    files = os.listdir(folder)
    images = list(filter(lambda f: f.endswith('.JPG') or f.endswith('.jpg'), files))
    xmls = list(filter(lambda f: f.endswith('.xml'), files))
    delete = []

    for img in images:
        filename = img[:img.rfind('.')] + '.xml'

        if filename not in xmls:
            delete.append(os.path.join(folder, img))

    for xml in xmls:
        a, b = (xml[:xml.rfind('.')] + '.jpg', xml[:xml.rfind('.')] + '.JPG')

        if a not in images and b not in images:
            delete.append(os.path.join(folder, xml))

    if delete:
        cmd = f'rm ' + ' '.join(delete)
        os.system(cmd)

        for d in delete:
            print('Deleted ' + d)
