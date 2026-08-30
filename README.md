# **LOINC Code Validator (Rust + Axum + HTMX)**

A small Rust web app that checks whether a LOINC code exists.  
No frontend frameworks, no noise - Axum, HTMX, Tailwind, and a call to the NIH API.

---

## **Local Setup**

```bash
cargo run
```

Open `http://127.0.0.1:3000`

That's the whole setup. No extra tooling.

---

## **How It Works**

- User enters a LOINC code  
- HTMX posts it to `/validate`  
- Backend calls the NIH Clinical Tables API  
- Response is a small HTML fragment: "valid" or "invalid"  
- HTMX swaps it into the page  
- Tailwind handles the colors

Everything stays server‑side. No SPA, no JS build chain.

---

## **Reflection: Strengths & Limitations**

### **Strengths**
- Backend is tiny and easy to reason about  
- Axum routing is straight to the point  
- HTMX keeps the UI chill and server‑side  
- Typed structs mean no JSON guessing games  
- Ships as one binary, no circus  
- Tests hit the usual edge cases  
- CI keeps things reproducible enough for a demo

### **Limitations**
Prototype vibes, not production vibes:

- Fully depends on the NIH API being awake  
- Tests hit the live API, so they wobble  
- No real validation layer (everything goes straight to the API)  
- Logic, API calls, and HTML are all glued together  
- No caching  
- No template engine (just `include_str!`)  
- Only handles one code at a time  
- Error handling is shallow — everything kinda collapses into "invalid"

Good enough for a small demo. Not something you'd ship to a hospital.

---

## **Tests**

Covers:

- Valid codes  
- Invalid codes  
- Empty input  
- Whitespace input  
- Missing `code` field (Axum 422)  
- Oversized input  
- Expected Tailwind classes in the HTML fragment

**AI‑assisted:**  
AI tossed me some edge‑case ideas (big inputs, broken forms, class checks).  

---

## **Possible Extensions**

- Split the repo at [loinc-validator-rs](https://github.com/JaskRendix/loinc-validator-rs) into a clean Rust workspace (core / CLI / web). Current layout works, but it's basically 3 projects sharing a fridge.  
- Batch validation (CSV upload, parallel checks)  
- Show LOINC metadata (version, synonyms, related terms)  
- Export results (JSON or CSV)  
- Small Dockerfile  
- Multi‑terminology mode (RxNorm, SNOMED, ICD‑10)  
- Suggest similar codes when the exact one isn't found  
- Semantic lookup with embeddings for messy hospital data  
- Offline mirror of the NIH dataset so the app doesn't die when the API does  
- Anomaly detector for weird LOINC usage patterns across batches  
- Temporal drift checks for sudden changes in code usage  
- Cross‑terminology consistency checks (LOINC vs ICD‑10 vs SNOMED)  
- LOINC neighborhood graph to explore related concepts  
- Unit sanity checker (catch nonsense like heart rate in kg)  
- Synthetic LOINC dataset generator for testing pipelines without real patient data  
- Cross‑hospital LOINC fingerprinting to spot unique usage patterns across different sites  
- LOINC anomaly heatmap to visualize hot zones of weird terminology behavior across a network  
- LOINC‑based synthetic patient generator using LOINC distributions to build fake patient profiles for testing

---

## **Design Notes**

- Placeholder code `6969-0` is just an example. Any valid LOINC code works.  
- HTMX + Tailwind via CDN keeps the frontend tiny.  
- HTML template loaded with `include_str!` to avoid embedding large HTML blocks in Rust.  
- `LoincResponse` matches the NIH API's 4‑element array.  
- Helper functions (`valid`, `invalid`, `error`) keep the handler readable.  
- Input is trimmed before validation.

---

## **AI Usage Notes**

**Human‑owned:**  
Backend logic, routing, Axum setup, API calls, typed structs, error handling, HTMX flow, all the architecture choices.

**AI‑assisted:**  
Got some wording tweaks, Tailwind class ideas, and a couple edge‑case test suggestions. Everything was reviewed and adjusted by hand.
