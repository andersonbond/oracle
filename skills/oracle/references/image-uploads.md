# Image uploads without unnecessary storage growth

Use this guide for uploaded photos, avatars, screenshots, and product images such as PNG and JPEG. Use an image optimizer to create appropriately sized display copies, and serve those copies in the UI. Choose original retention separately with the user. Optimization reduces avoidable bytes; quotas and lifecycle rules bound total storage growth.

## Let the user choose what happens to originals

Use an existing explicit original-storage policy when one is available. Otherwise, before implementing persistent uploads, ask the project owner whether unmodified originals are needed and where they should live. Record the chosen destination, access, retention, and deletion behavior in the project's existing design notes. Do not invent a provider, overwrite originals, or delete existing files to save space.

| Original policy | Storage decision |
| --- | --- |
| Keep only optimized copies in the application | Explain that the app cannot recover the full-resolution source. Let the uploader save the original locally if needed; do not assume a browser capture is already saved on their device. |
| Retain originals in application storage | Use a separate private location on durable storage, with its own quota and retention policy. Serve optimized copies for normal display. |
| Retain originals in a chosen object store or archive | Let the user choose an approved destination, access policy, and retention period. Keep originals separate from public delivery assets. |

If uploaders need an individual choice, expose the supported options and explain where each copy goes. A project-wide policy can be communicated without adding a storage chooser to every upload. Resolve the policy before accepting persistent uploads when no choice exists yet. Storage destinations must map to authorized locations, not arbitrary client-supplied filesystem paths or bucket names. Keep storage credentials out of browser code. Full-resolution source material needed for editing or evidence should remain unchanged in the chosen original store.

When originals are retained, preserve the source bytes before any client-side resize, compression, or metadata removal, and send those bytes to the selected original destination. A browser-generated display copy is not the raw original. Client-side optimization saves transfer bytes only for the copies it replaces; retaining the original remotely still requires transferring it.

## A small processing pipeline

1. **Validate and bound input.** Allow only the formats the feature supports. Check actual file content with a decoder rather than trusting a filename or browser MIME type. Enforce request limits while receiving bytes and configure decoder limits before decoding; bound dimensions, decoded pixels, frames, processing time, memory, and concurrency. Validate client-optimized copies again at the receiving service. See [OWASP's upload guidance](https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html).
2. **Create display copies.** Apply orientation before removing orientation metadata, resize to the intended display dimensions without upscaling, and encode at a quality appropriate to the content. Preserve transparency, legible text, and required color behavior. Explicitly support animation or reject it; do not silently discard frames. Remove unnecessary sensitive metadata such as GPS coordinates from served copies, while preserving the separately retained original according to policy.
3. **Measure the result.** Compare output bytes and visual quality. Already optimized images may not get smaller; avoid saving a larger redundant variant or claiming a reduction that did not occur. Any fallback must still satisfy validation, metadata, and storage policies; do not silently publish the raw upload on optimizer failure.
4. **Store and report truthfully.** Keep display copies and retained originals under distinct keys. Apply the intended access policy to both: optimizing a private image does not make its thumbnail public. Authorize upload, read, replacement, and deletion for the owning user or project. Use the existing suitable storage system, with durable local storage for a small single-server app when appropriate. Mark an upload complete only when all copies required by the chosen policy are stored; failures need an explicit retry or recoverable state. Do not leave temporary uploads indefinitely or delete the only required copy before its destination is confirmed.
5. **Bound retained data.** Set per-file and per-user/project quotas and limit generated variants. Account for originals, derivatives, temporary files, object versions, and backups in storage estimates. Clean abandoned uploads and superseded copies according to the chosen policy, with bounded retry cleanup. Keep uploaded files and their generated variants out of Git. Add queues, workers, or a CDN only when workload or delivery needs justify them.

## Choose an optimizer that fits the stack

| Context | Candidate and boundary |
| --- | --- |
| New Rust service or tool | The [`image` crate](https://docs.rs/image/latest/image/) provides decoding, image operations, and encoding. Check enabled codecs and encoder capabilities for the required formats. Configure [decoder limits](https://docs.rs/image/latest/image/struct.Limits.html); some allocation limits are best effort, so also bound the processing workload. |
| PNG compression in a Rust pipeline | [`oxipng`](https://github.com/oxipng/oxipng) optimizes PNG/APNG compression. It is not a JPEG optimizer or a replacement for resizing and upload validation. |
| Existing Node.js application | [`sharp`](https://sharp.pixelplumbing.com/api-output/) offers image output and metadata controls; use its [resize operations](https://sharp.pixelplumbing.com/api-resize/) when suitable. Keep the existing stack rather than adding another service just for language preference. |

These are candidates, not Oracle dependencies. Verify current APIs, supported codecs, toolchain/native requirements, and maintenance before choosing. PNG can suit transparency and crisp graphics; JPEG can suit photographs. Evaluate WebP or AVIF when quality, encoding cost, and consumer compatibility justify them. Do not force every upload into one format or generate every format and size.

A simple app can start with one bounded processing step, one useful display copy, and an optional thumbnail. Original storage remains the user's decision. Oracle adds no upload server, optimizer dependency, configuration table, or mandatory release gate.

## Verify the actual upload journey

Try representative photos and transparent/text-heavy PNGs, rotated images, a small already optimized file, and malformed or oversized input. Compare bytes, dimensions, quality, orientation, transparency, and served metadata. Test optimizer failure, a full quota, storage failure, retry, and abandoned-upload cleanup. Confirm that retained originals match the source bytes before client-side transformations, private originals and derivatives reject unauthorized access, and deletion follows the selected policy. Show useful progress and errors, and keep a failed upload retryable without reporting it as saved.

For a simple app, a few representative fixtures and one end-to-end upload can establish a useful starting point. Record measured savings and untested cases; never promise a universal compression ratio or unlimited storage. When evidence tracking is in scope, use the existing `TEST-001` workflow and applicable access-control checks, not a new image-specific gate.
