# Furnica 3

Executantul „Furnica” pentru învățarea algoritmilor, rescris în **Rust** după
programul original „Furnica” (Ant v2.00, 1999). Elevul scrie un program cu
comenzi simple, iar furnica îl execută pe o tabelă cu litere: se mișcă din
celulă în celulă și împinge literele din calea ei.

Fișierele `.ant` din versiunea veche se deschid și rulează fără modificări.
Furnica 3 salvează în același format, dar programul vechi înțelege doar
cuvintele-cheie fără diacritice (`INCEPUT`, `SFIRSIT`, `STINGA`…), tabele de
10 × 10 și blocuri neincluse unul în altul; compatibilitatea în acest sens nu a
fost încercată cu `furnica.exe`.

## Ce e nou față de versiunea originală

- interfață modernă, temă luminoasă și întunecată, siglă și iconiță noi;
- furnica e desenată vectorial: e clară la orice mărime, se rotește lin și își
  mișcă picioarele când merge;
- editor cu numerotarea rândurilor și evidențierea sintaxei;
- animație fluidă (furnica și litera împinsă alunecă), rândul care se execută
  evidențiat în editor, drumul furnicii desenat pe tabelă;
- execuție pas cu pas, pauză, viteză reglabilă sau execuție instantă;
- **analizor de cod**: programul e verificat în timp ce scrii, înainte de rulare
  (vezi mai jos);
- blocurile se pot pune oricum unul în altul (în original `REPETĂ` nu putea
  conține alt `REPETĂ`, iar `CÎT` alt `CÎT`);
- butoane care inserează comenzile (SUS, JOS, REPETĂ, CÎT, DACĂ, PROCEDURA…);
  procedura se inserează automat înainte de `ÎNCEPUT`;
- editarea tabelei direct pe ea, fără unelte de ales:
  - furnica se mută prin tragere (celula de destinație e verde sau roșie),
    iar un click pe ea o rotește;
  - literele se scriu de la tastatură, ca într-un text: click pe o celulă și
    scrieți, cursorul trece singur la celula următoare (Backspace, Delete,
    săgețile, Enter, Esc);
  - literele se mută prin tragere, peste altă literă fac schimb cu ea, iar
    trase în afara tabelei (sau cu click dreapta) se șterg;
  - tabela poate avea între 3 × 3 și 20 × 20 de celule;
- după rulare, tabela de pornire rămâne neschimbată; „Păstrează rezultatul”
  face din tabela obținută noua tabelă de pornire;
- interfața în română, rusă și engleză; cuvintele-cheie pot fi scrise
  în oricare dintre aceste limbi, cu sau fără diacritice;
- programul, tabela și setările se păstrează între porniri;
- deschiderea fișierelor prin drag & drop sau ca argument în linia de comandă.

## Pornire

Trebuie instalat [Rust](https://rustup.rs). Apoi, în acest dosar:

```sh
cargo run --release                  # pornește aplicația
cargo run --release -- program.ant   # deschide direct un fișier
```

Executabilul rezultat este în `target/release/` (`furnica.exe` pe Windows) și
poate fi copiat pe alte calculatoare: nu are nevoie de alte fișiere. Pe Windows
iconița furnicii este inclusă automat în `.exe`.

Pentru afișare se folosește OpenGL 2.0, ca să meargă și pe calculatoarele mai
vechi din laboratoare.

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

## Limbajul

| Comandă                                        | Ce face                                                   |
|------------------------------------------------|-----------------------------------------------------------|
| `SUS`, `JOS`, `STÎNGA`, `DREAPTA`              | mută furnica cu o celulă; o literă din cale este împinsă   |
| `ÎNCEPUT … SFÎRȘIT`                            | programul principal                                       |
| `REPETĂ N ORI … SFÎRȘIT REPETĂ`                | repetă de N ori                                           |
| `CÎT condiție EXECUTĂ … SFÎRȘIT CÎT`           | repetă cât timp condiția e adevărată                      |
| `DACĂ condiție ATUNCI comandă1 ALTFEL comandă2` | ramificare (ALTFEL poate lipsi)                          |
| `PROCEDURA nume … SFÎRȘIT PROCEDURA`           | definește o procedură (înainte de `ÎNCEPUT`)              |
| `nume`                                         | apelează procedura                                        |
| `// …`                                         | comentariu                                                |

Condiții: `E_MARGINE` (furnica privește spre marginea tabelei) și
`NU_E_MARGINE`. Furnica privește în direcția în care s-a mișcat ultima dată
(la început: direcția salvată în fișier, pe care o schimbă butonul „Rotire”).

Furnica nu poate ieși din tabelă, nu poate împinge o literă peste margine și
nu poate împinge două litere aflate una după alta. În oricare din aceste cazuri
programul se oprește cu eroare, iar locul e marcat cu roșu pe tabelă.

În engleză: `UP DOWN LEFT RIGHT BEGIN END REPEAT TIMES WHILE DO IF THEN ELSE
PROCEDURE BORDER NOT_BORDER`; în rusă: `ВВЕРХ ВНИЗ ВЛЕВО ВПРАВО НАЧАЛО КОНЕЦ
ПОВТОРИ РАЗ ПОКА СДЕЛАЙ ЕСЛИ ТОГДА ИНАЧЕ ПРОЦЕДУРА КРАЙ НЕ_КРАЙ` — exact
cuvintele din fișierele `romana.lng`, `engleza.lng` și `rusa.lng` ale
programului original.

Taste rapide: **F5** execută / pauză, **F6** pas cu pas, **Esc** stop,
**F7** verifică, **Ctrl+N / Ctrl+O / Ctrl+S** fișiere.

## Analizorul de cod

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

Erori detectate: cuvinte care nu sunt nici comenzi, nici proceduri, condiții
necunoscute, `REPETĂ` fără număr, blocuri neînchise sau închise greșit
(`SFÎRȘIT CÎT` în loc de `SFÎRȘIT REPETĂ`), `SFÎRȘIT`/`ALTFEL` rătăcite,
comandă lipsă după `ATUNCI`, `ÎNCEPUT` lipsă sau dublu, proceduri definite de
două ori sau scrise în programul principal, text după `SFÎRȘIT`.

Avertismente: proceduri nefolosite, blocuri goale, `REPETĂ 0 ORI`, cicluri
`CÎT` care nu mută furnica (pot rula la nesfârșit), proceduri care se
apelează pe ele însele fără oprire, program principal gol.

### Cum a fost reconstituit comportamentul original

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

## Structura codului

| Fișier              | Conținut                                                          |
|---------------------|-------------------------------------------------------------------|
| `src/lang.rs`       | lexer, parser tolerant la erori și compilator                     |
| `src/check.rs`      | analiza semantică: avertismente, proceduri, sugestii de corectură |
| `src/vm.rs`         | mașina virtuală care execută câte o mișcare pe rând               |
| `src/world.rs`      | tabela, literele, furnica și regulile de împingere                |
| `src/antfile.rs`    | citirea și scrierea fișierelor `.ant`, codificările vechi         |
| `src/table_edit.rs` | regulile editării tabelei: tragere, schimb, scriere de la tastatură |
| `src/app.rs`        | interfața grafică (egui / eframe)                                 |
| `src/i18n.rs`       | textele interfeței în română, rusă și engleză                     |
| `src/software_gl.rs`| repornirea cu OpenGL software (Mesa) pe Windows                   |
| `packaging/`        | arhivele pentru Linux, Windows 10/11 și Windows 7 (vezi `docs/releasing.md`) |
| `examples/`         | exemplele originale (`conferinta`, `svi99`) și cele noi           |
| `src/ant.rs`        | furnica desenată vectorial (rotire lină, picioare care pășesc)    |
| `assets/`           | fonturi, iconița (`icon.py` o redesenează după `src/ant.rs`)      |

Programul original e păstrat ca referință în dosarul `Furnica/` (ignorat de git).

## Release

Versiunea există într-un singur loc, `Cargo.toml`, și se schimbă cu
`packaging/version.sh set 3.1.0`. La un push pe ramura `release`, GitHub Actions
construiește arhivele pentru toate platformele și creează un **draft** de
release, care se publică manual. Pașii sunt descriși în
[`docs/releasing.md`](docs/releasing.md).

## Teste

```sh
cargo test
```

Testele verifică limbajul, formatul fișierelor, regulile de împingere,
rularea exemplelor (cu rezultatul exact al celor originale) și interfața:
apasă butoanele reale, trage furnica și literele și scrie pe tabelă, fără fereastră. Cu
`FURNICA_SHOTS=<dosar> cargo test` se salvează și capturi de ecran.

## Licență

Furnica 3 este software liber, distribuit sub licența
[GNU General Public License, versiunea 3](LICENSE) sau, la alegere, orice
versiune ulterioară.
