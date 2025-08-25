# Vy Changelog

> ⚠️ **PRE-ALPHA PROJECT** - Vy is in rapid development with frequent breaking changes. This changelog documents major architectural changes during pre-alpha development. No backward compatibility is maintained during this phase.

## Configuration System Overhaul - August 2025

### Major Changes to Configuration Approach

#### 🔐 Mandatory API Keys and Credentials

- All API keys and project IDs are now mandatory with no defaults
- Clear error messages guide users to set missing required fields

**Required fields:**

- `llm_api_key` - OpenAI API key for main AI responses
- `google_api_key` - Google API key for web search
- `google_search_engine_id` - Google Custom Search Engine ID
- `vector_memory.openai_api_key` - OpenAI API key for embeddings

#### 🤖 Hard-coded Model Defaults with Override Capability

- All models now use sensible hard-coded defaults
- Users can override any model setting as needed

**Model defaults:**

- `llm_model_id`: `gpt-4o-mini`
- `memory_model_id`: `gpt-4o-mini`
- `memory_similarity_model_id`: `gpt-4o-mini`
- `vector_memory.embedding_model`: `text-embedding-3-small`

#### 🔧 Enhanced Configuration Management

**Validation & Error Handling:**

- Automatic validation of required fields during config load
- Helpful error messages with specific guidance
- Clear distinction between mandatory and optional settings

**Improved Initialization:**

- `vy config init` now clearly separates mandatory from optional fields
- Better user experience with step-by-step guidance
- Links provided for obtaining required API keys

### Code Changes

#### Core Configuration (`vy-core/src/config.rs`)

- Added `validate_config()` function to check mandatory fields
- Updated default model from `gpt-3.5-turbo` to `gpt-4o-mini`
- Clear error messages for missing required configuration

#### Vector Memory (`vy-core/src/vector_memory.rs`)

- Added explicit default functions for all hard-coded values
- Made OpenAI API key mandatory (no empty string default)
- Used serde defaults for all configurable fields

#### CLI Configuration (`vy-cli/src/config.rs`)

- Added new config key: `VectorMemoryEmbeddingApiKey`
- Added `is_mandatory()` method to distinguish required fields
- Completely restructured `run_init()` for better UX:
  - Mandatory fields prompted first with clear labeling
  - Optional overrides grouped separately
  - Better guidance and explanations

#### Web Configuration (`vy-web/src/main.rs`)

- Made Google API fields mandatory via `get_required_env()`
- Added fallback from main API key to vector memory API key
- Maintains backward compatibility with environment variables

#### Documentation Updates

- Updated `README.md` with new configuration approach
- Created comprehensive `CONFIGURATION.md` guide
- Clear distinction between mandatory and optional settings
- Updated troubleshooting sections

### Setup

1. Run `vy config init` after installation
2. Follow prompts to set all required API keys
3. Optionally customize model settings (good defaults provided)

### Benefits

1. **Security**: No more empty defaults for sensitive credentials
2. **Simplicity**: Hard-coded sensible model defaults eliminate choice paralysis
3. **Clarity**: Clear distinction between what's required vs optional
4. **Reliability**: Validation ensures complete configuration before use
5. **User Experience**: Better error messages and initialization flow

### Technical Details

- Test configurations updated to match new requirements
- Build system unchanged - all changes are runtime configuration
- No changes to core AI functionality or memory systems

---

## Pre-Alpha Development Notes

**Development Philosophy:**

- **Breaking changes are expected** and implemented immediately without deprecation periods
- **Clean code prioritized** over backward compatibility during pre-alpha phase
- **Rapid iteration** preferred over extensive migration support
- **API stability** will come later during alpha/beta phases

**For Contributors:**

- All changes should prioritize code quality and maintainability
- Breaking existing interfaces is encouraged when it improves the system
- Focus on building the best possible foundation for future stability

---

_This changelog documents major changes during Vy's pre-alpha development phase, where breaking changes and API evolution are expected and welcomed._
