local llm = require('llm')

llm.setup({
  api_token = "REDACTED",
  model = "bigcode/starcoder", -- can be a model ID or an http(s) endpoint
  tokens_to_clear = { "<|endoftext|>" }, -- tokens to remove from the model's output
  -- parameters that are added to the request body
  query_params = {
    max_new_tokens = 200,
    temperature = 0.2,
    top_p = 0.95,
    stop_tokens = nil,
  },
  -- set this if the model supports fill in the middle
  fim = {
    enabled = true,
    prefix = "<fim_prefix>",
    middle = "<fim_middle>",
    suffix = "<fim_suffix>",
  },
  debounce_ms = 150,
  accept_keymap = "<leader>c",
  dismiss_keymap = "<S-Tab>",
  tls_skip_verify_insecure = false,
  -- llm-ls configuration, cf llm-ls section
  lsp = {
    bin_path = nil,
    version = "0.4.0",
  },
  tokenizer = nil, -- cf Tokenizer paragraph
  context_window = 8192, -- max number of tokens for the context window
  enable_suggestions_on_startup = true,
  enable_suggestions_on_files = "*", -- pattern matching syntax to enable suggestions on specific files, either a string or a list of strings
})
