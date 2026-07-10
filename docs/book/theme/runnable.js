// Every complete nevla example gets a Run button that executes it right
// on the page, go.dev style, via the playground's wasm module (served
// from /pkg/ on the same origin). The share link stays for editing.
(function () {
  let mod = null;
  async function ensure() {
    if (mod) return mod;
    const m = await import("/pkg/nevla_playground.js");
    await m.default();
    mod = m;
    return mod;
  }

  function attach(code) {
    const pre = code.closest("pre");
    if (!pre) return;
    const btn = document.createElement("button");
    btn.className = "nevla-run";
    btn.textContent = "Run";
    const out = document.createElement("pre");
    out.className = "nevla-out";
    out.style.display = "none";
    pre.insertAdjacentElement("afterend", out);
    pre.appendChild(btn);
    btn.addEventListener("click", async () => {
      btn.disabled = true;
      btn.textContent = "…";
      try {
        const m = await ensure();
        const r = m.run(code.textContent);
        out.style.display = "block";
        if (r.status === "ok") {
          out.classList.remove("err");
          out.textContent = r.stdout.length ? r.stdout : "(no output)";
        } else {
          out.classList.add("err");
          out.textContent =
            (r.stdout.length ? r.stdout + "\n" : "") + r.error;
        }
      } catch (e) {
        out.style.display = "block";
        out.classList.add("err");
        out.textContent = "could not load the playground runtime: " + e;
      }
      btn.disabled = false;
      btn.textContent = "Run";
    });
  }

  for (const code of document.querySelectorAll("code.language-nevla")) {
    if (code.textContent.includes("fn main")) {
      attach(code);
    }
  }
})();
