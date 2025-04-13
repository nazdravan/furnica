<div align="center">

<p align="center">
  <img src="docs/assets/furnica-mark.png" alt="Furnica" width="130">
</p>

<img src="docs/assets/furnica-banner.png" alt="Furnica" />


**Executantul „Furnica” pentru învățarea algoritmilor — elevul scrie un program cu comenzi simple, iar furnica îl execută pe o tabelă cu litere, împingând literele din calea ei.**

[![Licență](https://img.shields.io/badge/licen%C8%9B%C4%83-GPL--3.0-3b82f6?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-B7410E?style=flat-square&logo=rust&logoColor=white)](Cargo.toml)
[![egui](https://img.shields.io/badge/UI-egui%20%2F%20eframe-F97316?style=flat-square)](https://github.com/emilk/egui)
[![Windows](https://img.shields.io/badge/Windows-7%20%7C%2010%20%7C%2011-0078D6?style=flat-square&logo=windows&logoColor=white)](#windows-7-32-și-64-de-biți)
[![Linux](https://img.shields.io/badge/Linux-AppImage-FCC624?style=flat-square&logo=linux&logoColor=black)](packaging/linux/)
[![macOS](https://img.shields.io/badge/macOS-Apple%20Silicon%20%7C%20Intel-000000?style=flat-square&logo=apple&logoColor=white)](#macos)

[**Pornire rapidă**](#pornire-rapidă) · [Limbajul](#limbajul) · [Analizorul de cod](#analizorul-de-cod) · [Formatul fișierelor](#formatul-fișierelor) · [Releases](../../releases) · [Issues](../../issues)

</div>

<br />

Rescris în **Rust** după programul original „Furnica” (Ant v2.00, 1999).
Fișierele `.ant` din versiunea veche se deschid și rulează fără modificări.

<img src="docs/assets/screenshot.png" alt="Furnica: exemplul „conferinta” după execuție" />

## Pornire rapidă

Arhivele gata construite sunt pe pagina [Releases](../../releases):

```
furnica-<versiune>-linux-x86_64.AppImage         # Linux, un singur fișier
furnica-<versiune>-linux-x86_64.deb              # Ubuntu / Debian / Mint
furnica-<versiune>-linux-x86_64.tar.gz           # Linux, arhivă
furnica-<versiune>-macos-universal.dmg           # macOS (Apple Silicon și Intel)
furnica-<versiune>-windows-x64.msi               # Windows 10/11, 64 de biți, instalator
furnica-<versiune>-windows-x86.msi               # Windows 10/11, 32 de biți, instalator
furnica-<versiune>-windows-x64-portable.zip      # Windows 10/11, 64 de biți
furnica-<versiune>-windows-x86-portable.zip      # Windows 10/11, 32 de biți
furnica-<versiune>-windows7-x86-portable.zip     # Windows 7, 8, 10, 11 (32 și 64 de biți)
SHA256SUMS.txt                                   # sumele de control
```

Pe fiecare platformă:

- **Windows:** `.msi`-ul instalează Furnica în *Program Files*, cu scurtătură în
  meniul Start, iar fișierele `.ant` se deschid cu dublu-click (pentru mai
  multe calculatoare: `msiexec /i furnica-*.msi /qn`). Fără instalare: se
  dezarhivează zip-ul și se pornește `Furnica.exe`;
- **Linux:** AppImage-ul se face executabil și se pornește
  (`chmod +x furnica-*.AppImage && ./furnica-*.AppImage`); pe Ubuntu / Debian
  se poate instala `.deb`-ul (`sudo apt install ./furnica-*.deb`): Furnica
  apare în meniu, iar fișierele `.ant` se deschid cu dublu-click;
- **macOS:** se deschide `.dmg`-ul și `Furnica.app` se trage peste
  *Applications*. Nu e semnată cu un
  certificat Apple, deci la prima pornire: click dreapta → *Deschide*
  (pe macOS 15+: *Setări de sistem → Confidențialitate și securitate →
  Deschide oricum*), vezi [macOS](#macos).

### Din cod sursă

Trebuie instalat [Rust](https://rustup.rs). Apoi, în acest dosar:

```sh
cargo run --release                  # pornește aplicația
cargo run --release -- program.ant   # deschide direct un fișier
```

Executabilul rezultat este în `target/release/` (`furnica.exe` pe Windows) și
poate fi copiat pe alte calculatoare: nu are nevoie de alte fișiere. Pe Windows
iconița furnicii este inclusă automat în `.exe`. Pentru afișare se folosește
OpenGL 2.0, ca să meargă și pe calculatoarele mai vechi din laboratoare.

---

## Ce e nou față de versiunea originală

| | |
|---|---|
| **Interfață** | temă luminoasă și întunecată, siglă și iconiță noi; interfața în română, rusă și engleză |
| **Furnica** | desenată vectorial: clară la orice mărime, se rotește lin și își mișcă picioarele când merge |
| **Editor** | numerotarea rândurilor, evidențierea sintaxei, butoane care inserează comenzile (procedura se pune automat înainte de `ÎNCEPUT`) |
| **Execuție** | animație fluidă, rândul curent evidențiat, drumul furnicii desenat pe tabelă; pas cu pas, pauză, viteză reglabilă sau instant |
| **Analizor** | programul e verificat în timp ce scrii, cu sugestii de corectură ([detalii](#analizorul-de-cod)) |
| **Blocuri** | se pot pune oricum unul în altul (în original `REPETĂ` nu putea conține alt `REPETĂ`, iar `CÎT` alt `CÎT`) |
| **Tabela** | editată direct pe ea, între 3 × 3 și 20 × 20 de celule ([detalii](#editarea-tabelei)) |
| **Fișiere** | programul, tabela și setările se păstrează între porniri; deschidere prin drag & drop sau din linia de comandă |

### Editarea tabelei

<img src="docs/assets/editare.png" alt="Scrierea literelor pe tabelă" />

- **furnica** se mută prin tragere (celula de destinație e verde sau roșie),
  iar un click pe ea o rotește;
- **literele** se scriu de la tastatură, ca într-un text: click pe o celulă și
  scrieți, cursorul trece singur la celula următoare (Backspace, Delete,
  săgețile, Enter, Esc);
- literele se mută prin tragere; peste altă literă fac schimb cu ea, iar trase
  în afara tabelei (sau cu click dreapta) se șterg;
- după rulare, tabela de pornire rămâne neschimbată; **Păstrează rezultatul**
  face din tabela obținută noua tabelă de pornire.

---

## Limbajul

| Comandă | Ce face |
|---|---|
| `SUS`, `JOS`, `STÎNGA`, `DREAPTA` | mută furnica cu o celulă; o literă din cale este împinsă |
| `ÎNCEPUT … SFÎRȘIT` | programul principal |
| `REPETĂ N ORI … SFÎRȘIT REPETĂ` | repetă de N ori |
| `CÎT condiție EXECUTĂ … SFÎRȘIT CÎT` | repetă cât timp condiția e adevărată |
| `DACĂ condiție ATUNCI comandă1 ALTFEL comandă2` | ramificare (`ALTFEL` poate lipsi) |
| `PROCEDURA nume … SFÎRȘIT PROCEDURA` | definește o procedură (înainte de `ÎNCEPUT`) |
| `nume` | apelează procedura |
| `// …` | comentariu |

Condiții: `E_MARGINE` (furnica privește spre marginea tabelei) și
`NU_E_MARGINE`. Furnica privește în direcția în care s-a mișcat ultima dată
(la început: direcția salvată în fișier, pe care o schimbă butonul **Rotire**).

Furnica nu poate ieși din tabelă, nu poate împinge o literă peste margine și
nu poate împinge două litere aflate una după alta. În oricare din aceste cazuri
programul se oprește cu eroare, iar locul e marcat cu roșu pe tabelă.

### Cuvintele-cheie în trei limbi

Se pot scrie în oricare dintre limbi, cu sau fără diacritice — exact cuvintele
din fișierele `romana.lng`, `engleza.lng` și `rusa.lng` ale programului original.

| Română | English | Русский |
|---|---|---|
| `SUS JOS STÎNGA DREAPTA` | `UP DOWN LEFT RIGHT` | `ВВЕРХ ВНИЗ ВЛЕВО ВПРАВО` |
| `ÎNCEPUT SFÎRȘIT` | `BEGIN END` | `НАЧАЛО КОНЕЦ` |
| `REPETĂ ORI` | `REPEAT TIMES` | `ПОВТОРИ РАЗ` |
| `CÎT EXECUTĂ` | `WHILE DO` | `ПОКА СДЕЛАЙ` |
| `DACĂ ATUNCI ALTFEL` | `IF THEN ELSE` | `ЕСЛИ ТОГДА ИНАЧЕ` |
| `PROCEDURA` | `PROCEDURE` | `ПРОЦЕДУРА` |
| `E_MARGINE NU_E_MARGINE` | `BORDER NOT_BORDER` | `КРАЙ НЕ_КРАЙ` |

### Taste rapide

| Tastă | Acțiune |
|---|---|
| **F5** | execută / pauză |
| **F6** | pas cu pas |
| **Esc** | stop |
| **F7** | verifică |
| **Ctrl+N / Ctrl+O / Ctrl+S** | nou / deschide / salvează |

---

## Analizorul de cod

<img src="docs/assets/analizor.png" alt="Analizorul de cod cu erori și sugestii" />

Programul este analizat la fiecare modificare (se poate opri din *Setări*;
atunci analiza se face cu **Verifică** / F7). Analizorul nu se oprește la prima
greșeală, ci le găsește pe toate:

- cuvintele greșite sunt subliniate ondulat (roșu = eroare, galben = avertisment),
  iar rândul are un marcaj lângă număr; la hover apare explicația;
- lista **Probleme** de sub editor arată toate erorile și avertismentele;
  un click duce cursorul la cuvântul respectiv;
- pentru cuvinte scrise greșit se propune corectura („Poate ați vrut DREAPTA?”),
  aplicabilă cu butonul **Corectează** — și pentru nume de proceduri;
- programul cu erori nu pornește; avertismentele nu blochează rularea.

| Tip | Ce se detectează |
|---|---|
| **Erori** | cuvinte care nu sunt nici comenzi, nici proceduri; condiții necunoscute; `REPETĂ` fără număr; blocuri neînchise sau închise greșit (`SFÎRȘIT CÎT` în loc de `SFÎRȘIT REPETĂ`); `SFÎRȘIT`/`ALTFEL` rătăcite; comandă lipsă după `ATUNCI`; `ÎNCEPUT` lipsă sau dublu; proceduri definite de două ori sau scrise în programul principal; text după `SFÎRȘIT` |
| **Avertismente** | proceduri nefolosite; blocuri goale; `REPETĂ 0 ORI`; cicluri `CÎT` care nu mută furnica (pot rula la nesfârșit); proceduri care se apelează pe ele însele fără oprire; program principal gol |

---

## Compatibilitatea cu originalul

Regulile au fost luate din `furnica.exe` (Delphi 3): formularele din resurse
și codul mașină al comenzilor (programul vechi nu a fost rulat). Cu aceste
reguli, exemplele originale își formează exact cuvintele pe care le pregătesc:
„conferinta” și „svi99” pe primul rând (verificat în teste).

Decizii luate acolo unde comportamentul vechi era greșit sau nu era documentat:

- **`E_MARGINE` spre stînga și dreapta este corectat.** În original, după o
  mișcare spre dreapta se verifica marginea din *stînga* (și invers), așa că
  `CÎT NU_E_MARGINE EXECUTĂ DREAPTA` scotea furnica din tabelă. Sus și jos
  funcționau corect, iar exemplele originale folosesc doar `JOS` în cicluri,
  deci rulează la fel;
- un `*` pe celula furnicii din fișier este doar marcajul furnicii (așa îl
  scrie și programul vechi), nu o literă;
- la o eroare furnica rămâne pe loc, dar privește spre direcția în care a
  încercat să meargă, ca în original;
- `ATUNCI` și `EXECUTĂ` pot lipsi, iar `NU E_MARGINE` este acceptat ca
  `NU_E_MARGINE`; `EXECUTĂ nume` la început de rând apelează procedura.

Furnica 3 salvează în același format, dar programul vechi înțelege doar
cuvintele-cheie fără diacritice (`INCEPUT`, `SFIRSIT`, `STINGA`…), tabele de
10 × 10 și blocuri neincluse unul în altul; compatibilitatea în acest sens nu a
fost încercată cu `furnica.exe`.

---

## Formatul fișierelor

```text
Ant executor file
[table description]
1 10          poziția furnicii: coloana, rândul (de la 1)
3             direcția: 1 jos, 2 stînga, 3 sus, 4 dreapta
2 9 i         literele: coloana, rândul, litera
1 10 *        celula furnicii
666 666 *     sfârșitul tabelei
procedura …   programul, până la sfârșitul fișierului
```

Tabelele de altă dimensiune decât 10 × 10 au în plus rândul
`[size <coloane> <rânduri>]` după primul rând. Fișierele vechi (Windows-1250
sau Windows-1251) se recunosc automat; cele noi se salvează în UTF-8, cu CRLF.

---

## Windows 7 (32 și 64 de biți)

Un `.exe` construit normal cere Windows 10+ (cerință a Rust de la versiunea
1.78). Pentru laboratoarele cu Windows 7 există o variantă separată, construită
din Linux cu:

```sh
packaging/windows7/package.sh   # rezultatul: dist/release/furnica-<versiune>-windows7-x86-portable.zip
```

Scriptul instalează singur Rust nightly în `.cache/` și descarcă ce îi trebuie;
din sistem cere doar `curl`, `7z` și compilatorul MinGW pe 32 de biți
(`mingw-w64-gcc` pe Arch, `gcc-mingw-w64-i686` pe Debian/Ubuntu). Arhiva
merge pe Windows 7, 8, 10 și 11, pe 32 și pe 64 de biți, fără instalare și
fără actualizări suplimentare de Windows:

- folosește `msvcrt.dll` (există pe orice Windows 7), nu UCRT, care pe
  Windows 7 cere actualizarea KB2999226;
- `combase.dll` din arhivă (3,5 KB) redirecționează o funcție pe care
  bibliotecile Rust noi o caută acolo, dar care pe Windows 7 este în
  `ole32.dll`; pe Windows 8+ e ignorat;
- dosarul `soft-gl` conține Mesa (OpenGL software). Dacă placa video nu are
  driver cu OpenGL 2.0 (driver „Standard VGA”, mașini virtuale), programul se
  repornește singur cu Mesa. Mai lent, dar funcționează. Se poate forța cu
  `Furnica.exe --software-gl`;
- în această variantă lipsește doar suportul pentru cititoare de ecran
  (biblioteca respectivă cere Windows 8+). În cod e funcția `accessibility`,
  activă implicit în celelalte build-uri.

Este aceeași rețetă ca la Cangurul 3. **Atenție:** varianta nu a fost încă
pornită pe un Windows 7 real.

---

## macOS

Imaginea de disc `furnica-<versiune>-macos-universal.dmg` conține `Furnica.app`, un
singur program pentru Mac-urile cu Apple Silicon (M1…M4) și cu Intel, de la
macOS 10.13 în sus. Se construiește doar pe un Mac (în release, pe runner-ul
macOS din GitHub Actions):

```sh
packaging/macos/package.sh   # rezultatul: dist/release/furnica-<versiune>-macos-universal.dmg
```

Aplicația are doar o semnătură ad-hoc, nu una cu certificat Apple Developer,
așa că Gatekeeper o blochează la prima pornire. Se deschide cu click dreapta →
*Deschide* sau o dată din Terminal:

```sh
xattr -dr com.apple.quarantine /Applications/Furnica.app
```

Scurtăturile folosesc ⌘ în loc de Ctrl. **Atenție:** varianta se compilează
pentru ambele procesoare, dar nu a fost încă pornită pe un Mac real.

---

## Structura codului

| Fișier | Conținut |
|---|---|
| `src/lang.rs` | lexer, parser tolerant la erori și compilator |
| `src/check.rs` | analiza semantică: avertismente, proceduri, sugestii de corectură |
| `src/vm.rs` | mașina virtuală care execută câte o mișcare pe rând |
| `src/world.rs` | tabela, literele, furnica și regulile de împingere |
| `src/antfile.rs` | citirea și scrierea fișierelor `.ant`, codificările vechi |
| `src/table_edit.rs` | regulile editării tabelei: tragere, schimb, scriere de la tastatură |
| `src/ant.rs` | furnica desenată vectorial (rotire lină, picioare care pășesc) |
| `src/app.rs` | interfața grafică (egui / eframe) |
| `src/i18n.rs` | textele interfeței în română, rusă și engleză |
| `src/software_gl.rs` | repornirea cu OpenGL software (Mesa) pe Windows |
| `examples/` | exemplele originale (`conferinta`, `svi99`) și cele noi |
| `assets/` | fonturi, iconițele (`icon.py` le redesenează după `src/ant.rs`, inclusiv `furnica.icns` pentru macOS), `banner.py` pentru bannerul din README (`python3 assets/banner.py FURNICA -o docs/assets/furnica-banner.png`) |
| `packaging/` | AppImage, `.deb` și arhiva pentru Linux, `Furnica.app` pentru macOS, `.msi` și arhivele pentru Windows 10/11, arhiva pentru Windows 7 |

Programul original e păstrat ca referință în dosarul `Furnica/` (ignorat de git).

---

## Release

Versiunea există într-un singur loc, `Cargo.toml`, și se schimbă cu
`packaging/version.sh set 3.1.0`. Release-urile sunt **automate**: la un push
pe `main` cu o versiune încă nepublicată, GitHub Actions construiește pachetele
pentru toate platformele și publică release-ul. Un push cu aceeași versiune
rulează doar testele. Detalii în [`docs/releasing.md`](docs/releasing.md).

## Teste

```sh
cargo test
```

Testele verifică limbajul, formatul fișierelor, regulile de împingere,
rularea exemplelor (cu rezultatul exact al celor originale) și interfața:
apasă butoanele reale, trage furnica și literele și scrie pe tabelă, fără
fereastră. Cu `FURNICA_SHOTS=<dosar> cargo test` se salvează și capturi de ecran
(cele din `docs/assets/` sunt făcute așa).

## Licență

Furnica 3 este software liber, distribuit sub licența
[GNU General Public License, versiunea 3](LICENSE) sau, la alegere, orice
versiune ulterioară.
