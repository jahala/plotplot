# Recipes (things done rarely, so they stay instructions)

**A product frame.** Run the real command under a time limit with no terminal attached (`perl -e 'alarm 20; exec @ARGV' <command>`), in its human-readable format, and paste what it printed; never type a transcript. For a product that is a page, screenshot the real page in light and dark at desktop width and at phone width (under 520px). Title the frame with what ran, or "preview" while the product is still being built.

**A bed page.** Copy the brand's `.brand/reference/product.html` and change only its `swap:` spots: the bed's aliases in `:root` from its row in `colors.md` (its bloom; its word ink, or text with a bloom underline when it has none; the plate text ink or paper), its tiled mark, its place on the site plan in the direction sign, the hero and its frame, the proof band (four numbers counted from the repository at a named version, commit and date), its own sections, and the footer's identity column and current pill. Then run `petals check <page> --product <bed>`.

**Trying a heading face.** Load the face on a draft page and look. When the face has one weight and no italic, add `html{font-synthesis:none}` (`bash scripts/fix-typography.sh <page>` does it) and set its own weight, or the browser fakes bold and italic; the check reports any rule that asks a face for what it lacks.
