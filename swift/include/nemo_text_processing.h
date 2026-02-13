#ifndef NEMO_TEXT_PROCESSING_H
#define NEMO_TEXT_PROCESSING_H

#include <stdint.h>

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
 * Normalize a full sentence, replacing spoken-form spans with written form.
 *
 * Unlike nemo_normalize which expects the entire input to be a single expression,
 * this scans for normalizable spans within a larger sentence.
 *
 * @param input Null-terminated UTF-8 string
 * @return Newly allocated string, must be freed with nemo_free_string().
 */
char* nemo_normalize_sentence(const char* input);

/**
 * Normalize a full sentence with a configurable max span size.
 *
 * @param input Null-terminated UTF-8 string
 * @param max_span_tokens Maximum number of consecutive tokens per span (default 16)
 * @return Newly allocated string, must be freed with nemo_free_string().
 */
char* nemo_normalize_sentence_with_max_span(const char* input, uint32_t max_span_tokens);

/**
 * Add a custom spoken-to-written normalization rule.
 * Custom rules have the highest priority, checked before all built-in taggers.
 * If a rule with the same spoken form exists, it is replaced.
 *
 * @param spoken Null-terminated UTF-8 spoken form (case-insensitive)
 * @param written Null-terminated UTF-8 written form
 */
void nemo_add_rule(const char* spoken, const char* written);

/**
 * Remove a custom normalization rule.
 *
 * @param spoken Null-terminated UTF-8 spoken form to remove
 * @return 1 if the rule was found and removed, 0 otherwise
 */
int32_t nemo_remove_rule(const char* spoken);

/**
 * Clear all custom normalization rules.
 */
void nemo_clear_rules(void);

/**
 * Get the number of custom rules currently registered.
 */
uint32_t nemo_rule_count(void);

/**
 * Free a string allocated by nemo_normalize or nemo_normalize_sentence.
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
