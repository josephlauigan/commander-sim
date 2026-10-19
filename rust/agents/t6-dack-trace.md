# Brief: trace Dack Fayden first-flip piles (Tier 6, Stage 2)

Rules-tracing task. **Never run Python.** Don't commit. Edit only YOUR output file (named in your prompt).

1. Read `audit/t6/dack-fayden-white-stax/RULES-SHEET.md` in full: setup, the rules facts, outcome labels and the
   Oracle text. Use those facts. Don't fetch anything from the web; everything you need is in the sheet.
2. Create your output file at once with a header line, then trace your piles **one at a time**. After EACH pile,
   append its section to the file and save. A lead watches the file and stops an agent that writes nothing for 10
   minutes, so never hold more than one pile in your head.
3. For each pile write:
   ```
   ## Pile <n>: <A> + <B> + <C>  —  <OUTCOME LABEL>
   Trace: numbered steps — which triggers go on the stack (yours), the targets you choose, what enters or leaves and
   under whose control, until a loop is proven unbounded or the board stops changing. Short lines.
   Loop: the repeating cycle in one line, or "none".
   With a flicker spell: which of Ephemerate / Cloudshift / Touch the Spirit Realm would make it win, and how (one line).
   Second flip: if relevant, which of the remaining four creatures would complete a win, as "k of 4".
   Doubts: any rules point you weren't sure of.
   ```
4. Look for the BEST line for you in each pile (you choose targets and trigger order). Don't stop at the first line
   that fails.
5. When all your piles are done, append a summary table (pile, outcome, one-line reason) and finish. Final message:
   at most 8 lines, with the outcome counts.
