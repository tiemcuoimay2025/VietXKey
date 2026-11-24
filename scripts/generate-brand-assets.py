"""Export the original VietXKey keycap SVG to application and website assets.
Maintainer-only: Python Pillow, Node.js and sharp. Normal builds use committed assets.
"""
from pathlib import Path
import subprocess
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SVG = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" role="img" aria-label="VietXKey">
<defs>
 <linearGradient id="shell" x2="0" y2="1"><stop stop-color="#657e43"/><stop offset="1" stop-color="#344b28"/></linearGradient>
 <linearGradient id="face" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#c4d79e"/><stop offset=".52" stop-color="#a2bf75"/><stop offset="1" stop-color="#7e9f52"/></linearGradient>
 <filter id="shadow" x="-30%" y="-25%" width="160%" height="170%"><feDropShadow dx="0" dy="13" stdDeviation="11" flood-color="#26391c" flood-opacity=".22"/></filter>
</defs>
<g filter="url(#shadow)">
 <rect x="40" y="44" width="432" height="424" rx="92" fill="url(#shell)"/>
 <path d="M125 62H387Q448 62 448 123V345Q448 404 389 404H123Q64 404 64 345V123Q64 62 125 62Z" fill="url(#face)"/>
 <rect x="76" y="74" width="360" height="316" rx="64" fill="none" stroke="#f4f9e8" stroke-opacity=".38" stroke-width="3"/>
 <path d="M166 175L256 315L346 175" fill="none" stroke="#3c5a28" stroke-opacity=".22" stroke-width="45" stroke-linecap="round" stroke-linejoin="round"/>
 <path d="M166 170L256 310L346 170" fill="none" stroke="#f8faee" stroke-width="42" stroke-linecap="round" stroke-linejoin="round"/>
 <path d="M194 437H318" stroke="#a1bb7c" stroke-opacity=".6" stroke-width="9" stroke-linecap="round"/>
</g>
</svg>'''


def main():
    assets = ROOT / 'assets'
    web = ROOT / 'site/assets'
    web.mkdir(parents=True, exist_ok=True)
    svg_path = assets / 'VietXKey.svg'
    svg_path.write_text(SVG, encoding='utf-8')
    render = "require('sharp')(process.argv[1]).resize(512,512).png().toFile(process.argv[2]).catch(e=>{console.error(e);process.exit(1)})"
    for name, svg in [('VietXKey', SVG), ('VietXKey-TSF', SVG.replace('</svg>', '<circle cx="420" cy="408" r="50" fill="#f4f8e9" stroke="#486333" stroke-width="9"/><g fill="#486333"><circle cx="398" cy="408" r="6"/><circle cx="420" cy="408" r="6"/><circle cx="442" cy="408" r="6"/></g></svg>'))]:
        input_file = svg_path if name == 'VietXKey' else assets / '.vietxkey-tsf-export.svg'
        if input_file != svg_path:
            input_file.write_text(svg, encoding='utf-8')
        png = assets / (name + '.png')
        subprocess.run(['node', '-e', render, str(input_file), str(png)], check=True)
        with Image.open(png) as image:
            image.save(assets / (name + '.ico'), format='ICO', sizes=[(s,s) for s in [16,20,24,28,32,40,48,64,128,256]])
        if input_file != svg_path:
            input_file.unlink()
    (web / 'keycap.svg').write_text(SVG, encoding='utf-8')
    (web / 'keycap.png').write_bytes((assets / 'VietXKey.png').read_bytes())
    (web / 'favicon.ico').write_bytes((assets / 'VietXKey.ico').read_bytes())
    print('Exported keycap SVG, PNG and multi-resolution ICO assets.')

if __name__ == '__main__':
    main()
