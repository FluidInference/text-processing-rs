#ifndef NEMO_TEXT_PROCESSING_H
#define NEMO_TEXT_PROCESSING_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Normalize spoken-form text to written form.
 *
 * @param input Null-terminated UTF-8 string of spoken text
 * @return Newly allocated string with written form, or NULL on error.
 *         Must be freed with nemo_free_string().
 *
 * Example:
 *   char* result = nemo_normalize("two hundred");
 *   // result is "200"
 *   nemo_free_string(result);
 */
char* nemo_normalize(const char* input);

/**
 * Free a string allocated by nemo_normalize.
 *
 * @param s Pointer returned by nemo_normalize, or NULL (no-op)
 */
void nemo_free_string(char* s);

/**
 * Get the library version.
 *
 * @return Static version string, do not free.
 */
const char* nemo_version(void);

#ifdef __cplusplus
}
#endif

#endif /* NEMO_TEXT_PROCESSING_H */
