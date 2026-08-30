new url:
    #!/usr/bin/env python3
    import html, os, re, subprocess, urllib.request
    from urllib.parse import urljoin

    url = "{{url}}"
    headers = {"User-Agent": "Mozilla/5.0"}
    with urllib.request.urlopen(urllib.request.Request(url, headers=headers)) as resp:
        page = resp.read().decode("utf-8")

    stem = os.path.splitext(os.path.basename(url))[0]
    number, name_part = stem.split(".", 1)
    number = f"{int(number):02d}"
    name_part = name_part.lower().replace("_", "-")
    slug = f"{number}-{name_part}"
    name = " ".join(w.capitalize() for w in name_part.split("-"))
    title = f"{number} {name}"

    def section(section_id):
        m = re.search(rf'<section id="{section_id}">(.*?)</section>', page, re.S)
        return m.group(1) if m else ""

    def clean(text):
        return html.unescape(re.sub(r"<[^>]+>", "", text)).strip()

    description = "\n\n".join(clean(p) for p in re.findall(r"<p>(.*?)</p>", section("description"), re.S))
    specification = "\n".join(f"- {clean(li)}" for li in re.findall(r"<li><p>(.*?)</p></li>", section("specification"), re.S))
    img_src = re.search(r'src="([^"]+)"', section("connect")).group(1)

    img_url = urljoin(url, img_src)
    tmp_img = f"/tmp/{slug}{os.path.splitext(img_src)[1]}"
    with urllib.request.urlopen(urllib.request.Request(img_url, headers=headers)) as resp, open(tmp_img, "wb") as f:
        f.write(resp.read())
    subprocess.run(["sips", "-s", "format", "jpeg", tmp_img, "--out", f"docs/src/images/{slug}.jpg"], check=True, capture_output=True)
    os.remove(tmp_img)

    module_path = f"docs/src/modules/{slug}.md"
    content = "\n".join([
        f"# {title}", "",
        "## Description", "",
        description, "",
        "## Specification", "",
        specification, "",
        "## Connect", "",
        f"![Image](../images/{slug}.jpg)", "",
        "## Code", "",
        "## References", "",
        f"- [Hosyond 45 in 1 Sensor Kit Documentation]({url})", "",
    ])
    with open(module_path, "w") as f:
        f.write(content)

    with open("docs/src/SUMMARY.md", "a") as f:
        f.write(f"    - [{title}](./modules/{slug}.md)\n")

    with open("docs/src/modules.md", "a") as f:
        f.write(f"{number}. [{name}](./modules/{slug}.md)\n")

    print(f"Created {module_path}")
