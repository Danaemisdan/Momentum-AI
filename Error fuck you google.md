Screencast error: send failed because receiver is gone
Screencast error: send failed because receiver is gone
-> [Engine] Stopped: Hit EOS token
[EXEC] Action: UpdateMemory { key: "user_talked", data: Bool(true) }
-> [Memory] Store: user_talked
Screencast error: send failed because receiver is gone
Screencast error: send failed because receiver is gone
--- STEP 8 ---
--> [Session Lock] Active goal: Can you find me some jobs?
-> [Conversational] No browser capture (Chrome stays hidden).
Vision Engine: bypassed (preventing hallucination/latency until Qwen2-VL is integrated).
Engine prompt assembled: 2953 bytes
Engine prompt: 2953 bytes → 687 tokens (KV Cache Match: 606 tokens)
Screencast error: send failed because receiver is gone
^C
🛑 Shutdown signal received. Cleaning up Chrome...
✅ Chrome terminated cleanly.
Momentum shut down.
-> [Engine] Stopped: stream_tx channel closed
zsh: segmentation fault  cargo run --release
sanjeevn@Sanjeevs-MacBook-Pro momentum-agent % 
sanjeevn@Sanjeevs-MacBook-Pro momentum-agent % clear

sanjeevn@Sanjeevs-MacBook-Pro momentum-agent % cd "/Users/sanjeevn/Downloads/Momentum AI/momentum-agent" && cargo run --release

   Compiling momentum-agent v0.1.0 (/Users/sanjeevn/Downloads/Momentum AI/momentum-agent)
warning: variable does not need to be mutable
   --> src/main.rs:472:25
    |
472 |             if let Some(mut last_s) = session_memory.last_state.take() {
    |                         ----^^^^^^
    |                         |
    |                         help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
   --> src/main.rs:859:16
    |
859 |             Ok(mut response) => {
    |                ----^^^^^^^^
    |                |
    |                help: remove this `mut`

warning: value assigned to `ui_match` is never read
   --> src/skills.rs:300:32
    |
300 |             let mut ui_match = 0.0;
    |                                ^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `skill_menu`
   --> src/main.rs:700:13
    |
700 |         let skill_menu = state.vault.get_skill_menu().await;
    |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_skill_menu`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: methods `extract_semantic_dom` and `capture_viewport_screenshot` are never used
   --> src/browser.rs:800:18
    |
 29 | impl MomentumBrowser {
    | -------------------- methods in this implementation
...
800 |     pub async fn extract_semantic_dom(&self) -> Result<String, Box<dyn ...
    |                  ^^^^^^^^^^^^^^^^^^^^
...
886 |     pub async fn capture_viewport_screenshot(&self) -> Result<Vec<u8>, ...
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: methods `get_platform_yaml_for_url` and `load_brain_context` are never used
   --> src/skills.rs:385:18
    |
110 | impl SkillVaultRouter {
    | --------------------- methods in this implementation
...
385 |     pub async fn get_platform_yaml_for_url(&self, url: &str) -> Option<...
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^
...
465 |     pub fn load_brain_context(&self) -> String {
    |            ^^^^^^^^^^^^^^^^^^

warning: field `is_recovery` is never read
   --> src/types.rs:100:9
    |
 87 | pub struct Decision {
    |            -------- field in this struct
...
100 |     pub is_recovery: bool,
    |         ^^^^^^^^^^^
    |
    = note: `Decision` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: function `all_matches` is never used
  --> src/patterns.rs:98:8
   |
98 | pub fn all_matches<'a>(matches: &'a [PatternMatch], pattern_type: &str) ...
   |        ^^^^^^^^^^^

warning: multiple variants are never constructed
  --> src/brain.rs:32:5
   |
27 | pub enum Intent {
   |          ------ variants in this enum
...
32 |     OpenNewTab     { url: String },
   |     ^^^^^^^^^^
...
54 |     ClickCoords    { x: f64, y: f64 },
   |     ^^^^^^^^^^^
55 |     Type           { anchor_hint: String, value: String },
   |     ^^^^
...
60 |     WatchVideo     { query: String },
   |     ^^^^^^^^^^
...
63 |     FillForm       { fields: Vec<(String, String)> },
   |     ^^^^^^^^
...
70 |     SwitchTab      { hint: String },
   |     ^^^^^^^^^
...
73 |     Remember       { key: String, value: String },
   |     ^^^^^^^^
74 |     Recall         { key: String },
   |     ^^^^^^
...
77 |     TaskDone,
   |     ^^^^^^^^
   |
   = note: `Intent` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: `momentum-agent` (bin "momentum-agent") generated 9 warnings (run `cargo fix --bin "momentum-agent" -p momentum-agent` to apply 3 suggestions)
    Finished `release` profile [optimized] target(s) in 11.31s
     Running `target/release/momentum-agent`
Launching Momentum stealth browser...
-> Momentum Chrome profile: /Users/sanjeevn/.momentum/chrome-profile
   -> Cleared stale lock: SingletonSocket
ggml_metal_device_init: tensor API disabled for pre-M5 and pre-A19 devices
ggml_metal_library_init: using embedded metal library
ggml_metal_library_init: loaded in 0.027 sec
ggml_metal_rsets_init: creating a residency set collection (keep_alive = 180 s)
ggml_metal_device_init: GPU name:   MTL0
ggml_metal_device_init: GPU family: MTLGPUFamilyApple7  (1007)
ggml_metal_device_init: GPU family: MTLGPUFamilyCommon3 (3003)
ggml_metal_device_init: GPU family: MTLGPUFamilyMetal3  (5001)
ggml_metal_device_init: simdgroup reduction   = true
ggml_metal_device_init: simdgroup matrix mul. = true
ggml_metal_device_init: has unified memory    = true
ggml_metal_device_init: has bfloat            = true
ggml_metal_device_init: has tensor            = false
ggml_metal_device_init: use residency sets    = true
ggml_metal_device_init: use shared buffers    = true
ggml_metal_device_init: recommendedMaxWorkingSetSize  = 11453.25 MB
llama_model_load_from_file_impl: using device MTL0 (Apple M1 Pro) (unknown id) - 10922 MiB free
llama_model_loader: loaded meta data with 35 key-value pairs and 255 tensors from /Users/sanjeevn/Downloads/Momentum AI/momentum-engine-3b.gguf (version GGUF V3 (latest))
llama_model_loader: Dumping metadata keys/values. Note: KV overrides do not apply in this output.
llama_model_loader: - kv   0:                       general.architecture str              = llama
llama_model_loader: - kv   1:                               general.type str              = model
llama_model_loader: - kv   2:                               general.name str              = Llama 3.2 3B Instruct
llama_model_loader: - kv   3:                           general.finetune str              = Instruct
llama_model_loader: - kv   4:                           general.basename str              = Llama-3.2
llama_model_loader: - kv   5:                         general.size_label str              = 3B
llama_model_loader: - kv   6:                            general.license str              = llama3.2
llama_model_loader: - kv   7:                               general.tags arr[str,6]       = ["facebook", "meta", "pytorch", "llam...
llama_model_loader: - kv   8:                          general.languages arr[str,8]       = ["en", "de", "fr", "it", "pt", "hi", ...
llama_model_loader: - kv   9:                          llama.block_count u32              = 28
llama_model_loader: - kv  10:                       llama.context_length u32              = 131072
llama_model_loader: - kv  11:                     llama.embedding_length u32              = 3072
llama_model_loader: - kv  12:                  llama.feed_forward_length u32              = 8192
llama_model_loader: - kv  13:                 llama.attention.head_count u32              = 24
llama_model_loader: - kv  14:              llama.attention.head_count_kv u32              = 8
llama_model_loader: - kv  15:                       llama.rope.freq_base f32              = 500000.000000
llama_model_loader: - kv  16:     llama.attention.layer_norm_rms_epsilon f32              = 0.000010
llama_model_loader: - kv  17:                 llama.attention.key_length u32              = 128
llama_model_loader: - kv  18:               llama.attention.value_length u32              = 128
llama_model_loader: - kv  19:                          general.file_type u32              = 17
llama_model_loader: - kv  20:                           llama.vocab_size u32              = 128256
llama_model_loader: - kv  21:                 llama.rope.dimension_count u32              = 128
llama_model_loader: - kv  22:                       tokenizer.ggml.model str              = gpt2
llama_model_loader: - kv  23:                         tokenizer.ggml.pre str              = llama-bpe
llama_model_loader: - kv  24:                      tokenizer.ggml.tokens arr[str,128256]  = ["!", "\"", "#", "$", "%", "&", "'", ...
llama_model_loader: - kv  25:                  tokenizer.ggml.token_type arr[i32,128256]  = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, ...
llama_model_loader: - kv  26:                      tokenizer.ggml.merges arr[str,280147]  = ["Ġ Ġ", "Ġ ĠĠĠ", "ĠĠ ĠĠ", "...
llama_model_loader: - kv  27:                tokenizer.ggml.bos_token_id u32              = 128000
llama_model_loader: - kv  28:                tokenizer.ggml.eos_token_id u32              = 128009
llama_model_loader: - kv  29:                    tokenizer.chat_template str              = {{- bos_token }}\n{%- if custom_tools ...
llama_model_loader: - kv  30:               general.quantization_version u32              = 2
llama_model_loader: - kv  31:                      quantize.imatrix.file str              = /models_out/Llama-3.2-3B-Instruct-GGU...
llama_model_loader: - kv  32:                   quantize.imatrix.dataset str              = /training_dir/calibration_datav3.txt
llama_model_loader: - kv  33:             quantize.imatrix.entries_count i32              = 196
llama_model_loader: - kv  34:              quantize.imatrix.chunks_count i32              = 125
llama_model_loader: - type  f32:   58 tensors
llama_model_loader: - type q5_K:  168 tensors
llama_model_loader: - type q6_K:   29 tensors
print_info: file format = GGUF V3 (latest)
print_info: file type   = Q5_K - Medium
print_info: file size   = 2.16 GiB (5.76 BPW) 
init_tokenizer: initializing tokenizer for type 2
load: 0 unused tokens
load: control token: 128254 '<|reserved_special_token_246|>' is not marked as EOG
load: control token: 128252 '<|reserved_special_token_244|>' is not marked as EOG
load: control token: 128251 '<|reserved_special_token_243|>' is not marked as EOG
load: control token: 128250 '<|reserved_special_token_242|>' is not marked as EOG
load: control token: 128248 '<|reserved_special_token_240|>' is not marked as EOG
load: control token: 128247 '<|reserved_special_token_239|>' is not marked as EOG
load: control token: 128245 '<|reserved_special_token_237|>' is not marked as EOG
load: control token: 128244 '<|reserved_special_token_236|>' is not marked as EOG
load: control token: 128243 '<|reserved_special_token_235|>' is not marked as EOG
load: control token: 128240 '<|reserved_special_token_232|>' is not marked as EOG
load: control token: 128238 '<|reserved_special_token_230|>' is not marked as EOG
load: control token: 128235 '<|reserved_special_token_227|>' is not marked as EOG
load: control token: 128234 '<|reserved_special_token_226|>' is not marked as EOG
load: control token: 128229 '<|reserved_special_token_221|>' is not marked as EOG
load: control token: 128227 '<|reserved_special_token_219|>' is not marked as EOG
load: control token: 128226 '<|reserved_special_token_218|>' is not marked as EOG
load: control token: 128224 '<|reserved_special_token_216|>' is not marked as EOG
load: control token: 128223 '<|reserved_special_token_215|>' is not marked as EOG
load: control token: 128221 '<|reserved_special_token_213|>' is not marked as EOG
load: control token: 128219 '<|reserved_special_token_211|>' is not marked as EOG
load: control token: 128218 '<|reserved_special_token_210|>' is not marked as EOG
load: control token: 128217 '<|reserved_special_token_209|>' is not marked as EOG
load: control token: 128216 '<|reserved_special_token_208|>' is not marked as EOG
load: control token: 128215 '<|reserved_special_token_207|>' is not marked as EOG
load: control token: 128213 '<|reserved_special_token_205|>' is not marked as EOG
load: control token: 128211 '<|reserved_special_token_203|>' is not marked as EOG
load: control token: 128210 '<|reserved_special_token_202|>' is not marked as EOG
load: control token: 128209 '<|reserved_special_token_201|>' is not marked as EOG
load: control token: 128208 '<|reserved_special_token_200|>' is not marked as EOG
load: control token: 128207 '<|reserved_special_token_199|>' is not marked as EOG
load: control token: 128204 '<|reserved_special_token_196|>' is not marked as EOG
load: control token: 128202 '<|reserved_special_token_194|>' is not marked as EOG
load: control token: 128197 '<|reserved_special_token_189|>' is not marked as EOG
load: control token: 128195 '<|reserved_special_token_187|>' is not marked as EOG
load: control token: 128194 '<|reserved_special_token_186|>' is not marked as EOG
load: control token: 128191 '<|reserved_special_token_183|>' is not marked as EOG
load: control token: 128190 '<|reserved_special_token_182|>' is not marked as EOG
load: control token: 128188 '<|reserved_special_token_180|>' is not marked as EOG
load: control token: 128187 '<|reserved_special_token_179|>' is not marked as EOG
load: control token: 128185 '<|reserved_special_token_177|>' is not marked as EOG
load: control token: 128184 '<|reserved_special_token_176|>' is not marked as EOG
load: control token: 128183 '<|reserved_special_token_175|>' is not marked as EOG
load: control token: 128178 '<|reserved_special_token_170|>' is not marked as EOG
load: control token: 128177 '<|reserved_special_token_169|>' is not marked as EOG
load: control token: 128176 '<|reserved_special_token_168|>' is not marked as EOG
load: control token: 128175 '<|reserved_special_token_167|>' is not marked as EOG
load: control token: 128174 '<|reserved_special_token_166|>' is not marked as EOG
load: control token: 128173 '<|reserved_special_token_165|>' is not marked as EOG
load: control token: 128172 '<|reserved_special_token_164|>' is not marked as EOG
load: control token: 128169 '<|reserved_special_token_161|>' is not marked as EOG
load: control token: 128167 '<|reserved_special_token_159|>' is not marked as EOG
load: control token: 128166 '<|reserved_special_token_158|>' is not marked as EOG
load: control token: 128160 '<|reserved_special_token_152|>' is not marked as EOG
load: control token: 128159 '<|reserved_special_token_151|>' is not marked as EOG
load: control token: 128157 '<|reserved_special_token_149|>' is not marked as EOG
load: control token: 128156 '<|reserved_special_token_148|>' is not marked as EOG
load: control token: 128154 '<|reserved_special_token_146|>' is not marked as EOG
load: control token: 128152 '<|reserved_special_token_144|>' is not marked as EOG
load: control token: 128151 '<|reserved_special_token_143|>' is not marked as EOG
load: control token: 128150 '<|reserved_special_token_142|>' is not marked as EOG
load: control token: 128147 '<|reserved_special_token_139|>' is not marked as EOG
load: control token: 128144 '<|reserved_special_token_136|>' is not marked as EOG
load: control token: 128142 '<|reserved_special_token_134|>' is not marked as EOG
load: control token: 128141 '<|reserved_special_token_133|>' is not marked as EOG
load: control token: 128140 '<|reserved_special_token_132|>' is not marked as EOG
load: control token: 128133 '<|reserved_special_token_125|>' is not marked as EOG
load: control token: 128130 '<|reserved_special_token_122|>' is not marked as EOG
load: control token: 128128 '<|reserved_special_token_120|>' is not marked as EOG
load: control token: 128127 '<|reserved_special_token_119|>' is not marked as EOG
load: control token: 128126 '<|reserved_special_token_118|>' is not marked as EOG
load: control token: 128125 '<|reserved_special_token_117|>' is not marked as EOG
load: control token: 128124 '<|reserved_special_token_116|>' is not marked as EOG
load: control token: 128123 '<|reserved_special_token_115|>' is not marked as EOG
load: control token: 128122 '<|reserved_special_token_114|>' is not marked as EOG
load: control token: 128121 '<|reserved_special_token_113|>' is not marked as EOG
load: control token: 128120 '<|reserved_special_token_112|>' is not marked as EOG
load: control token: 128119 '<|reserved_special_token_111|>' is not marked as EOG
load: control token: 128116 '<|reserved_special_token_108|>' is not marked as EOG
load: control token: 128115 '<|reserved_special_token_107|>' is not marked as EOG
load: control token: 128114 '<|reserved_special_token_106|>' is not marked as EOG
load: control token: 128113 '<|reserved_special_token_105|>' is not marked as EOG
load: control token: 128111 '<|reserved_special_token_103|>' is not marked as EOG
load: control token: 128110 '<|reserved_special_token_102|>' is not marked as EOG
load: control token: 128107 '<|reserved_special_token_99|>' is not marked as EOG
load: control token: 128106 '<|reserved_special_token_98|>' is not marked as EOG
load: control token: 128105 '<|reserved_special_token_97|>' is not marked as EOG
load: control token: 128104 '<|reserved_special_token_96|>' is not marked as EOG
load: control token: 128103 '<|reserved_special_token_95|>' is not marked as EOG
load: control token: 128100 '<|reserved_special_token_92|>' is not marked as EOG
load: control token: 128097 '<|reserved_special_token_89|>' is not marked as EOG
load: control token: 128096 '<|reserved_special_token_88|>' is not marked as EOG
load: control token: 128094 '<|reserved_special_token_86|>' is not marked as EOG
load: control token: 128093 '<|reserved_special_token_85|>' is not marked as EOG
load: control token: 128090 '<|reserved_special_token_82|>' is not marked as EOG
load: control token: 128089 '<|reserved_special_token_81|>' is not marked as EOG
load: control token: 128087 '<|reserved_special_token_79|>' is not marked as EOG
load: control token: 128085 '<|reserved_special_token_77|>' is not marked as EOG
load: control token: 128080 '<|reserved_special_token_72|>' is not marked as EOG
load: control token: 128077 '<|reserved_special_token_69|>' is not marked as EOG
load: control token: 128076 '<|reserved_special_token_68|>' is not marked as EOG
load: control token: 128073 '<|reserved_special_token_65|>' is not marked as EOG
load: control token: 128070 '<|reserved_special_token_62|>' is not marked as EOG
load: control token: 128069 '<|reserved_special_token_61|>' is not marked as EOG
load: control token: 128067 '<|reserved_special_token_59|>' is not marked as EOG
load: control token: 128064 '<|reserved_special_token_56|>' is not marked as EOG
load: control token: 128062 '<|reserved_special_token_54|>' is not marked as EOG
load: control token: 128061 '<|reserved_special_token_53|>' is not marked as EOG
load: control token: 128060 '<|reserved_special_token_52|>' is not marked as EOG
load: control token: 128054 '<|reserved_special_token_46|>' is not marked as EOG
load: control token: 128045 '<|reserved_special_token_37|>' is not marked as EOG
load: control token: 128044 '<|reserved_special_token_36|>' is not marked as EOG
load: control token: 128043 '<|reserved_special_token_35|>' is not marked as EOG
load: control token: 128042 '<|reserved_special_token_34|>' is not marked as EOG
load: control token: 128038 '<|reserved_special_token_30|>' is not marked as EOG
load: control token: 128037 '<|reserved_special_token_29|>' is not marked as EOG
load: control token: 128035 '<|reserved_special_token_27|>' is not marked as EOG
load: control token: 128034 '<|reserved_special_token_26|>' is not marked as EOG
load: control token: 128033 '<|reserved_special_token_25|>' is not marked as EOG
load: control token: 128032 '<|reserved_special_token_24|>' is not marked as EOG
load: control token: 128030 '<|reserved_special_token_22|>' is not marked as EOG
load: control token: 128029 '<|reserved_special_token_21|>' is not marked as EOG
load: control token: 128028 '<|reserved_special_token_20|>' is not marked as EOG
load: control token: 128026 '<|reserved_special_token_18|>' is not marked as EOG
load: control token: 128025 '<|reserved_special_token_17|>' is not marked as EOG
load: control token: 128024 '<|reserved_special_token_16|>' is not marked as EOG
load: control token: 128022 '<|reserved_special_token_14|>' is not marked as EOG
load: control token: 128020 '<|reserved_special_token_12|>' is not marked as EOG
load: control token: 128017 '<|reserved_special_token_9|>' is not marked as EOG
load: control token: 128016 '<|reserved_special_token_8|>' is not marked as EOG
load: control token: 128015 '<|reserved_special_token_7|>' is not marked as EOG
load: control token: 128014 '<|reserved_special_token_6|>' is not marked as EOG
load: control token: 128013 '<|reserved_special_token_5|>' is not marked as EOG
load: control token: 128011 '<|reserved_special_token_3|>' is not marked as EOG
load: control token: 128010 '<|python_tag|>' is not marked as EOG
load: control token: 128006 '<|start_header_id|>' is not marked as EOG
load: control token: 128003 '<|reserved_special_token_1|>' is not marked as EOG
load: control token: 128002 '<|reserved_special_token_0|>' is not marked as EOG
load: control token: 128000 '<|begin_of_text|>' is not marked as EOG
load: control token: 128041 '<|reserved_special_token_33|>' is not marked as EOG
load: control token: 128063 '<|reserved_special_token_55|>' is not marked as EOG
load: control token: 128046 '<|reserved_special_token_38|>' is not marked as EOG
load: control token: 128007 '<|end_header_id|>' is not marked as EOG
load: control token: 128065 '<|reserved_special_token_57|>' is not marked as EOG
load: control token: 128171 '<|reserved_special_token_163|>' is not marked as EOG
load: control token: 128162 '<|reserved_special_token_154|>' is not marked as EOG
load: control token: 128165 '<|reserved_special_token_157|>' is not marked as EOG
load: control token: 128057 '<|reserved_special_token_49|>' is not marked as EOG
load: control token: 128050 '<|reserved_special_token_42|>' is not marked as EOG
load: control token: 128056 '<|reserved_special_token_48|>' is not marked as EOG
load: control token: 128230 '<|reserved_special_token_222|>' is not marked as EOG
load: control token: 128098 '<|reserved_special_token_90|>' is not marked as EOG
load: control token: 128153 '<|reserved_special_token_145|>' is not marked as EOG
load: control token: 128084 '<|reserved_special_token_76|>' is not marked as EOG
load: control token: 128082 '<|reserved_special_token_74|>' is not marked as EOG
load: control token: 128102 '<|reserved_special_token_94|>' is not marked as EOG
load: control token: 128253 '<|reserved_special_token_245|>' is not marked as EOG
load: control token: 128179 '<|reserved_special_token_171|>' is not marked as EOG
load: control token: 128071 '<|reserved_special_token_63|>' is not marked as EOG
load: control token: 128135 '<|reserved_special_token_127|>' is not marked as EOG
load: control token: 128161 '<|reserved_special_token_153|>' is not marked as EOG
load: control token: 128164 '<|reserved_special_token_156|>' is not marked as EOG
load: control token: 128134 '<|reserved_special_token_126|>' is not marked as EOG
load: control token: 128249 '<|reserved_special_token_241|>' is not marked as EOG
load: control token: 128004 '<|finetune_right_pad_id|>' is not marked as EOG
load: control token: 128036 '<|reserved_special_token_28|>' is not marked as EOG
load: control token: 128148 '<|reserved_special_token_140|>' is not marked as EOG
load: control token: 128181 '<|reserved_special_token_173|>' is not marked as EOG
load: control token: 128222 '<|reserved_special_token_214|>' is not marked as EOG
load: control token: 128075 '<|reserved_special_token_67|>' is not marked as EOG
load: control token: 128241 '<|reserved_special_token_233|>' is not marked as EOG
load: control token: 128051 '<|reserved_special_token_43|>' is not marked as EOG
load: control token: 128068 '<|reserved_special_token_60|>' is not marked as EOG
load: control token: 128149 '<|reserved_special_token_141|>' is not marked as EOG
load: control token: 128201 '<|reserved_special_token_193|>' is not marked as EOG
load: control token: 128058 '<|reserved_special_token_50|>' is not marked as EOG
load: control token: 128146 '<|reserved_special_token_138|>' is not marked as EOG
load: control token: 128143 '<|reserved_special_token_135|>' is not marked as EOG
load: control token: 128023 '<|reserved_special_token_15|>' is not marked as EOG
load: control token: 128039 '<|reserved_special_token_31|>' is not marked as EOG
load: control token: 128132 '<|reserved_special_token_124|>' is not marked as EOG
load: control token: 128101 '<|reserved_special_token_93|>' is not marked as EOG
load: control token: 128212 '<|reserved_special_token_204|>' is not marked as EOG
load: control token: 128189 '<|reserved_special_token_181|>' is not marked as EOG
load: control token: 128225 '<|reserved_special_token_217|>' is not marked as EOG
load: control token: 128129 '<|reserved_special_token_121|>' is not marked as EOG
load: control token: 128005 '<|reserved_special_token_2|>' is not marked as EOG
load: control token: 128078 '<|reserved_special_token_70|>' is not marked as EOG
load: control token: 128163 '<|reserved_special_token_155|>' is not marked as EOG
load: control token: 128072 '<|reserved_special_token_64|>' is not marked as EOG
load: control token: 128112 '<|reserved_special_token_104|>' is not marked as EOG
load: control token: 128186 '<|reserved_special_token_178|>' is not marked as EOG
load: control token: 128095 '<|reserved_special_token_87|>' is not marked as EOG
load: control token: 128109 '<|reserved_special_token_101|>' is not marked as EOG
load: control token: 128099 '<|reserved_special_token_91|>' is not marked as EOG
load: control token: 128138 '<|reserved_special_token_130|>' is not marked as EOG
load: control token: 128193 '<|reserved_special_token_185|>' is not marked as EOG
load: control token: 128199 '<|reserved_special_token_191|>' is not marked as EOG
load: control token: 128048 '<|reserved_special_token_40|>' is not marked as EOG
load: control token: 128088 '<|reserved_special_token_80|>' is not marked as EOG
load: control token: 128192 '<|reserved_special_token_184|>' is not marked as EOG
load: control token: 128136 '<|reserved_special_token_128|>' is not marked as EOG
load: control token: 128092 '<|reserved_special_token_84|>' is not marked as EOG
load: control token: 128158 '<|reserved_special_token_150|>' is not marked as EOG
load: control token: 128049 '<|reserved_special_token_41|>' is not marked as EOG
load: control token: 128031 '<|reserved_special_token_23|>' is not marked as EOG
load: control token: 128255 '<|reserved_special_token_247|>' is not marked as EOG
load: control token: 128182 '<|reserved_special_token_174|>' is not marked as EOG
load: control token: 128066 '<|reserved_special_token_58|>' is not marked as EOG
load: control token: 128180 '<|reserved_special_token_172|>' is not marked as EOG
load: control token: 128233 '<|reserved_special_token_225|>' is not marked as EOG
load: control token: 128079 '<|reserved_special_token_71|>' is not marked as EOG
load: control token: 128081 '<|reserved_special_token_73|>' is not marked as EOG
load: control token: 128231 '<|reserved_special_token_223|>' is not marked as EOG
load: control token: 128196 '<|reserved_special_token_188|>' is not marked as EOG
load: control token: 128047 '<|reserved_special_token_39|>' is not marked as EOG
load: control token: 128083 '<|reserved_special_token_75|>' is not marked as EOG
load: control token: 128139 '<|reserved_special_token_131|>' is not marked as EOG
load: control token: 128131 '<|reserved_special_token_123|>' is not marked as EOG
load: control token: 128118 '<|reserved_special_token_110|>' is not marked as EOG
load: control token: 128053 '<|reserved_special_token_45|>' is not marked as EOG
load: control token: 128220 '<|reserved_special_token_212|>' is not marked as EOG
load: control token: 128108 '<|reserved_special_token_100|>' is not marked as EOG
load: control token: 128091 '<|reserved_special_token_83|>' is not marked as EOG
load: control token: 128203 '<|reserved_special_token_195|>' is not marked as EOG
load: control token: 128059 '<|reserved_special_token_51|>' is not marked as EOG
load: control token: 128019 '<|reserved_special_token_11|>' is not marked as EOG
load: control token: 128170 '<|reserved_special_token_162|>' is not marked as EOG
load: control token: 128205 '<|reserved_special_token_197|>' is not marked as EOG
load: control token: 128040 '<|reserved_special_token_32|>' is not marked as EOG
load: control token: 128200 '<|reserved_special_token_192|>' is not marked as EOG
load: control token: 128236 '<|reserved_special_token_228|>' is not marked as EOG
load: control token: 128145 '<|reserved_special_token_137|>' is not marked as EOG
load: control token: 128168 '<|reserved_special_token_160|>' is not marked as EOG
load: control token: 128214 '<|reserved_special_token_206|>' is not marked as EOG
load: control token: 128137 '<|reserved_special_token_129|>' is not marked as EOG
load: control token: 128232 '<|reserved_special_token_224|>' is not marked as EOG
load: control token: 128239 '<|reserved_special_token_231|>' is not marked as EOG
load: control token: 128055 '<|reserved_special_token_47|>' is not marked as EOG
load: control token: 128228 '<|reserved_special_token_220|>' is not marked as EOG
load: control token: 128206 '<|reserved_special_token_198|>' is not marked as EOG
load: control token: 128018 '<|reserved_special_token_10|>' is not marked as EOG
load: control token: 128012 '<|reserved_special_token_4|>' is not marked as EOG
load: control token: 128198 '<|reserved_special_token_190|>' is not marked as EOG
load: control token: 128021 '<|reserved_special_token_13|>' is not marked as EOG
load: control token: 128086 '<|reserved_special_token_78|>' is not marked as EOG
load: control token: 128074 '<|reserved_special_token_66|>' is not marked as EOG
load: control token: 128027 '<|reserved_special_token_19|>' is not marked as EOG
load: control token: 128242 '<|reserved_special_token_234|>' is not marked as EOG
load: control token: 128155 '<|reserved_special_token_147|>' is not marked as EOG
load: control token: 128052 '<|reserved_special_token_44|>' is not marked as EOG
load: control token: 128246 '<|reserved_special_token_238|>' is not marked as EOG
load: control token: 128117 '<|reserved_special_token_109|>' is not marked as EOG
load: control token: 128237 '<|reserved_special_token_229|>' is not marked as EOG
load: printing all EOG tokens:
load:   - 128001 ('<|end_of_text|>')
load:   - 128008 ('<|eom_id|>')
load:   - 128009 ('<|eot_id|>')
load: special tokens cache size = 256
load: token to piece cache size = 0.7999 MB
print_info: arch                  = llama
print_info: vocab_only            = 0
print_info: no_alloc              = 0
print_info: n_ctx_train           = 131072
print_info: n_embd                = 3072
print_info: n_embd_inp            = 3072
print_info: n_layer               = 28
print_info: n_head                = 24
print_info: n_head_kv             = 8
print_info: n_rot                 = 128
print_info: n_swa                 = 0
print_info: is_swa_any            = 0
print_info: n_embd_head_k         = 128
print_info: n_embd_head_v         = 128
print_info: n_gqa                 = 3
print_info: n_embd_k_gqa          = 1024
print_info: n_embd_v_gqa          = 1024
print_info: f_norm_eps            = 0.0e+00
print_info: f_norm_rms_eps        = 1.0e-05
print_info: f_clamp_kqv           = 0.0e+00
print_info: f_max_alibi_bias      = 0.0e+00
print_info: f_logit_scale         = 0.0e+00
print_info: f_attn_scale          = 0.0e+00
print_info: n_ff                  = 8192
print_info: n_expert              = 0
print_info: n_expert_used         = 0
print_info: n_expert_groups       = 0
print_info: n_group_used          = 0
print_info: causal attn           = 1
print_info: pooling type          = 0
print_info: rope type             = 0
print_info: rope scaling          = linear
print_info: freq_base_train       = 500000.0
print_info: freq_scale_train      = 1
print_info: n_ctx_orig_yarn       = 131072
print_info: rope_yarn_log_mul     = 0.0000
print_info: rope_finetuned        = unknown
print_info: model type            = 3B
print_info: model params          = 3.21 B
print_info: general.name          = Llama 3.2 3B Instruct
print_info: vocab type            = BPE
print_info: n_vocab               = 128256
print_info: n_merges              = 280147
print_info: BOS token             = 128000 '<|begin_of_text|>'
print_info: EOS token             = 128009 '<|eot_id|>'
print_info: EOT token             = 128001 '<|end_of_text|>'
print_info: EOM token             = 128008 '<|eom_id|>'
print_info: LF token              = 198 'Ċ'
print_info: EOG token             = 128001 '<|end_of_text|>'
print_info: EOG token             = 128008 '<|eom_id|>'
print_info: EOG token             = 128009 '<|eot_id|>'
print_info: max token length      = 256
load_tensors: loading model tensors, this can take a while... (mmap = true, direct_io = false)
load_tensors: layer   0 assigned to device MTL0, is_swa = 0
load_tensors: layer   1 assigned to device MTL0, is_swa = 0
load_tensors: layer   2 assigned to device MTL0, is_swa = 0
load_tensors: layer   3 assigned to device MTL0, is_swa = 0
load_tensors: layer   4 assigned to device MTL0, is_swa = 0
load_tensors: layer   5 assigned to device MTL0, is_swa = 0
load_tensors: layer   6 assigned to device MTL0, is_swa = 0
load_tensors: layer   7 assigned to device MTL0, is_swa = 0
load_tensors: layer   8 assigned to device MTL0, is_swa = 0
load_tensors: layer   9 assigned to device MTL0, is_swa = 0
load_tensors: layer  10 assigned to device MTL0, is_swa = 0
load_tensors: layer  11 assigned to device MTL0, is_swa = 0
load_tensors: layer  12 assigned to device MTL0, is_swa = 0
load_tensors: layer  13 assigned to device MTL0, is_swa = 0
load_tensors: layer  14 assigned to device MTL0, is_swa = 0
load_tensors: layer  15 assigned to device MTL0, is_swa = 0
load_tensors: layer  16 assigned to device MTL0, is_swa = 0
load_tensors: layer  17 assigned to device MTL0, is_swa = 0
load_tensors: layer  18 assigned to device MTL0, is_swa = 0
load_tensors: layer  19 assigned to device MTL0, is_swa = 0
load_tensors: layer  20 assigned to device MTL0, is_swa = 0
load_tensors: layer  21 assigned to device MTL0, is_swa = 0
load_tensors: layer  22 assigned to device MTL0, is_swa = 0
load_tensors: layer  23 assigned to device MTL0, is_swa = 0
load_tensors: layer  24 assigned to device MTL0, is_swa = 0
load_tensors: layer  25 assigned to device MTL0, is_swa = 0
load_tensors: layer  26 assigned to device MTL0, is_swa = 0
load_tensors: layer  27 assigned to device MTL0, is_swa = 0
load_tensors: layer  28 assigned to device MTL0, is_swa = 0
create_tensor: loading tensor token_embd.weight
create_tensor: loading tensor output_norm.weight
create_tensor: loading tensor token_embd.weight
create_tensor: loading tensor blk.0.attn_norm.weight
create_tensor: loading tensor blk.0.attn_q.weight
create_tensor: loading tensor blk.0.attn_k.weight
create_tensor: loading tensor blk.0.attn_v.weight
create_tensor: loading tensor blk.0.attn_output.weight
create_tensor: loading tensor blk.0.ffn_norm.weight
create_tensor: loading tensor rope_freqs.weight
create_tensor: loading tensor blk.0.ffn_gate.weight
create_tensor: loading tensor blk.0.ffn_down.weight
create_tensor: loading tensor blk.0.ffn_up.weight
create_tensor: loading tensor blk.1.attn_norm.weight
create_tensor: loading tensor blk.1.attn_q.weight
create_tensor: loading tensor blk.1.attn_k.weight
create_tensor: loading tensor blk.1.attn_v.weight
create_tensor: loading tensor blk.1.attn_output.weight
create_tensor: loading tensor blk.1.ffn_norm.weight
create_tensor: loading tensor blk.1.ffn_gate.weight
create_tensor: loading tensor blk.1.ffn_down.weight
create_tensor: loading tensor blk.1.ffn_up.weight
create_tensor: loading tensor blk.2.attn_norm.weight
create_tensor: loading tensor blk.2.attn_q.weight
create_tensor: loading tensor blk.2.attn_k.weight
create_tensor: loading tensor blk.2.attn_v.weight
create_tensor: loading tensor blk.2.attn_output.weight
create_tensor: loading tensor blk.2.ffn_norm.weight
create_tensor: loading tensor blk.2.ffn_gate.weight
create_tensor: loading tensor blk.2.ffn_down.weight
create_tensor: loading tensor blk.2.ffn_up.weight
create_tensor: loading tensor blk.3.attn_norm.weight
create_tensor: loading tensor blk.3.attn_q.weight
create_tensor: loading tensor blk.3.attn_k.weight
create_tensor: loading tensor blk.3.attn_v.weight
create_tensor: loading tensor blk.3.attn_output.weight
create_tensor: loading tensor blk.3.ffn_norm.weight
create_tensor: loading tensor blk.3.ffn_gate.weight
create_tensor: loading tensor blk.3.ffn_down.weight
create_tensor: loading tensor blk.3.ffn_up.weight
create_tensor: loading tensor blk.4.attn_norm.weight
create_tensor: loading tensor blk.4.attn_q.weight
create_tensor: loading tensor blk.4.attn_k.weight
create_tensor: loading tensor blk.4.attn_v.weight
create_tensor: loading tensor blk.4.attn_output.weight
create_tensor: loading tensor blk.4.ffn_norm.weight
create_tensor: loading tensor blk.4.ffn_gate.weight
create_tensor: loading tensor blk.4.ffn_down.weight
create_tensor: loading tensor blk.4.ffn_up.weight
create_tensor: loading tensor blk.5.attn_norm.weight
create_tensor: loading tensor blk.5.attn_q.weight
create_tensor: loading tensor blk.5.attn_k.weight
create_tensor: loading tensor blk.5.attn_v.weight
create_tensor: loading tensor blk.5.attn_output.weight
create_tensor: loading tensor blk.5.ffn_norm.weight
create_tensor: loading tensor blk.5.ffn_gate.weight
create_tensor: loading tensor blk.5.ffn_down.weight
create_tensor: loading tensor blk.5.ffn_up.weight
create_tensor: loading tensor blk.6.attn_norm.weight
create_tensor: loading tensor blk.6.attn_q.weight
create_tensor: loading tensor blk.6.attn_k.weight
create_tensor: loading tensor blk.6.attn_v.weight
create_tensor: loading tensor blk.6.attn_output.weight
create_tensor: loading tensor blk.6.ffn_norm.weight
create_tensor: loading tensor blk.6.ffn_gate.weight
create_tensor: loading tensor blk.6.ffn_down.weight
create_tensor: loading tensor blk.6.ffn_up.weight
create_tensor: loading tensor blk.7.attn_norm.weight
create_tensor: loading tensor blk.7.attn_q.weight
create_tensor: loading tensor blk.7.attn_k.weight
create_tensor: loading tensor blk.7.attn_v.weight
create_tensor: loading tensor blk.7.attn_output.weight
create_tensor: loading tensor blk.7.ffn_norm.weight
create_tensor: loading tensor blk.7.ffn_gate.weight
create_tensor: loading tensor blk.7.ffn_down.weight
create_tensor: loading tensor blk.7.ffn_up.weight
create_tensor: loading tensor blk.8.attn_norm.weight
create_tensor: loading tensor blk.8.attn_q.weight
create_tensor: loading tensor blk.8.attn_k.weight
create_tensor: loading tensor blk.8.attn_v.weight
create_tensor: loading tensor blk.8.attn_output.weight
create_tensor: loading tensor blk.8.ffn_norm.weight
create_tensor: loading tensor blk.8.ffn_gate.weight
create_tensor: loading tensor blk.8.ffn_down.weight
create_tensor: loading tensor blk.8.ffn_up.weight
create_tensor: loading tensor blk.9.attn_norm.weight
create_tensor: loading tensor blk.9.attn_q.weight
create_tensor: loading tensor blk.9.attn_k.weight
create_tensor: loading tensor blk.9.attn_v.weight
create_tensor: loading tensor blk.9.attn_output.weight
create_tensor: loading tensor blk.9.ffn_norm.weight
create_tensor: loading tensor blk.9.ffn_gate.weight
create_tensor: loading tensor blk.9.ffn_down.weight
create_tensor: loading tensor blk.9.ffn_up.weight
create_tensor: loading tensor blk.10.attn_norm.weight
create_tensor: loading tensor blk.10.attn_q.weight
create_tensor: loading tensor blk.10.attn_k.weight
create_tensor: loading tensor blk.10.attn_v.weight
create_tensor: loading tensor blk.10.attn_output.weight
create_tensor: loading tensor blk.10.ffn_norm.weight
create_tensor: loading tensor blk.10.ffn_gate.weight
create_tensor: loading tensor blk.10.ffn_down.weight
create_tensor: loading tensor blk.10.ffn_up.weight
create_tensor: loading tensor blk.11.attn_norm.weight
create_tensor: loading tensor blk.11.attn_q.weight
create_tensor: loading tensor blk.11.attn_k.weight
create_tensor: loading tensor blk.11.attn_v.weight
create_tensor: loading tensor blk.11.attn_output.weight
create_tensor: loading tensor blk.11.ffn_norm.weight
create_tensor: loading tensor blk.11.ffn_gate.weight
create_tensor: loading tensor blk.11.ffn_down.weight
create_tensor: loading tensor blk.11.ffn_up.weight
create_tensor: loading tensor blk.12.attn_norm.weight
create_tensor: loading tensor blk.12.attn_q.weight
create_tensor: loading tensor blk.12.attn_k.weight
create_tensor: loading tensor blk.12.attn_v.weight
create_tensor: loading tensor blk.12.attn_output.weight
create_tensor: loading tensor blk.12.ffn_norm.weight
create_tensor: loading tensor blk.12.ffn_gate.weight
create_tensor: loading tensor blk.12.ffn_down.weight
create_tensor: loading tensor blk.12.ffn_up.weight
create_tensor: loading tensor blk.13.attn_norm.weight
create_tensor: loading tensor blk.13.attn_q.weight
create_tensor: loading tensor blk.13.attn_k.weight
create_tensor: loading tensor blk.13.attn_v.weight
create_tensor: loading tensor blk.13.attn_output.weight
create_tensor: loading tensor blk.13.ffn_norm.weight
create_tensor: loading tensor blk.13.ffn_gate.weight
create_tensor: loading tensor blk.13.ffn_down.weight
create_tensor: loading tensor blk.13.ffn_up.weight
create_tensor: loading tensor blk.14.attn_norm.weight
create_tensor: loading tensor blk.14.attn_q.weight
create_tensor: loading tensor blk.14.attn_k.weight
create_tensor: loading tensor blk.14.attn_v.weight
create_tensor: loading tensor blk.14.attn_output.weight
create_tensor: loading tensor blk.14.ffn_norm.weight
create_tensor: loading tensor blk.14.ffn_gate.weight
create_tensor: loading tensor blk.14.ffn_down.weight
create_tensor: loading tensor blk.14.ffn_up.weight
create_tensor: loading tensor blk.15.attn_norm.weight
create_tensor: loading tensor blk.15.attn_q.weight
create_tensor: loading tensor blk.15.attn_k.weight
create_tensor: loading tensor blk.15.attn_v.weight
create_tensor: loading tensor blk.15.attn_output.weight
create_tensor: loading tensor blk.15.ffn_norm.weight
create_tensor: loading tensor blk.15.ffn_gate.weight
create_tensor: loading tensor blk.15.ffn_down.weight
create_tensor: loading tensor blk.15.ffn_up.weight
create_tensor: loading tensor blk.16.attn_norm.weight
create_tensor: loading tensor blk.16.attn_q.weight
create_tensor: loading tensor blk.16.attn_k.weight
create_tensor: loading tensor blk.16.attn_v.weight
create_tensor: loading tensor blk.16.attn_output.weight
create_tensor: loading tensor blk.16.ffn_norm.weight
create_tensor: loading tensor blk.16.ffn_gate.weight
create_tensor: loading tensor blk.16.ffn_down.weight
create_tensor: loading tensor blk.16.ffn_up.weight
create_tensor: loading tensor blk.17.attn_norm.weight
create_tensor: loading tensor blk.17.attn_q.weight
create_tensor: loading tensor blk.17.attn_k.weight
create_tensor: loading tensor blk.17.attn_v.weight
create_tensor: loading tensor blk.17.attn_output.weight
create_tensor: loading tensor blk.17.ffn_norm.weight
create_tensor: loading tensor blk.17.ffn_gate.weight
create_tensor: loading tensor blk.17.ffn_down.weight
create_tensor: loading tensor blk.17.ffn_up.weight
create_tensor: loading tensor blk.18.attn_norm.weight
create_tensor: loading tensor blk.18.attn_q.weight
create_tensor: loading tensor blk.18.attn_k.weight
create_tensor: loading tensor blk.18.attn_v.weight
create_tensor: loading tensor blk.18.attn_output.weight
create_tensor: loading tensor blk.18.ffn_norm.weight
create_tensor: loading tensor blk.18.ffn_gate.weight
create_tensor: loading tensor blk.18.ffn_down.weight
create_tensor: loading tensor blk.18.ffn_up.weight
create_tensor: loading tensor blk.19.attn_norm.weight
create_tensor: loading tensor blk.19.attn_q.weight
create_tensor: loading tensor blk.19.attn_k.weight
create_tensor: loading tensor blk.19.attn_v.weight
create_tensor: loading tensor blk.19.attn_output.weight
create_tensor: loading tensor blk.19.ffn_norm.weight
create_tensor: loading tensor blk.19.ffn_gate.weight
create_tensor: loading tensor blk.19.ffn_down.weight
create_tensor: loading tensor blk.19.ffn_up.weight
create_tensor: loading tensor blk.20.attn_norm.weight
create_tensor: loading tensor blk.20.attn_q.weight
create_tensor: loading tensor blk.20.attn_k.weight
create_tensor: loading tensor blk.20.attn_v.weight
create_tensor: loading tensor blk.20.attn_output.weight
create_tensor: loading tensor blk.20.ffn_norm.weight
create_tensor: loading tensor blk.20.ffn_gate.weight
create_tensor: loading tensor blk.20.ffn_down.weight
create_tensor: loading tensor blk.20.ffn_up.weight
create_tensor: loading tensor blk.21.attn_norm.weight
create_tensor: loading tensor blk.21.attn_q.weight
create_tensor: loading tensor blk.21.attn_k.weight
create_tensor: loading tensor blk.21.attn_v.weight
create_tensor: loading tensor blk.21.attn_output.weight
create_tensor: loading tensor blk.21.ffn_norm.weight
create_tensor: loading tensor blk.21.ffn_gate.weight
create_tensor: loading tensor blk.21.ffn_down.weight
create_tensor: loading tensor blk.21.ffn_up.weight
create_tensor: loading tensor blk.22.attn_norm.weight
create_tensor: loading tensor blk.22.attn_q.weight
create_tensor: loading tensor blk.22.attn_k.weight
create_tensor: loading tensor blk.22.attn_v.weight
create_tensor: loading tensor blk.22.attn_output.weight
create_tensor: loading tensor blk.22.ffn_norm.weight
create_tensor: loading tensor blk.22.ffn_gate.weight
create_tensor: loading tensor blk.22.ffn_down.weight
create_tensor: loading tensor blk.22.ffn_up.weight
create_tensor: loading tensor blk.23.attn_norm.weight
create_tensor: loading tensor blk.23.attn_q.weight
create_tensor: loading tensor blk.23.attn_k.weight
create_tensor: loading tensor blk.23.attn_v.weight
create_tensor: loading tensor blk.23.attn_output.weight
create_tensor: loading tensor blk.23.ffn_norm.weight
create_tensor: loading tensor blk.23.ffn_gate.weight
create_tensor: loading tensor blk.23.ffn_down.weight
create_tensor: loading tensor blk.23.ffn_up.weight
create_tensor: loading tensor blk.24.attn_norm.weight
create_tensor: loading tensor blk.24.attn_q.weight
create_tensor: loading tensor blk.24.attn_k.weight
create_tensor: loading tensor blk.24.attn_v.weight
create_tensor: loading tensor blk.24.attn_output.weight
create_tensor: loading tensor blk.24.ffn_norm.weight
create_tensor: loading tensor blk.24.ffn_gate.weight
create_tensor: loading tensor blk.24.ffn_down.weight
create_tensor: loading tensor blk.24.ffn_up.weight
create_tensor: loading tensor blk.25.attn_norm.weight
create_tensor: loading tensor blk.25.attn_q.weight
create_tensor: loading tensor blk.25.attn_k.weight
create_tensor: loading tensor blk.25.attn_v.weight
create_tensor: loading tensor blk.25.attn_output.weight
create_tensor: loading tensor blk.25.ffn_norm.weight
create_tensor: loading tensor blk.25.ffn_gate.weight
create_tensor: loading tensor blk.25.ffn_down.weight
create_tensor: loading tensor blk.25.ffn_up.weight
create_tensor: loading tensor blk.26.attn_norm.weight
create_tensor: loading tensor blk.26.attn_q.weight
create_tensor: loading tensor blk.26.attn_k.weight
create_tensor: loading tensor blk.26.attn_v.weight
create_tensor: loading tensor blk.26.attn_output.weight
create_tensor: loading tensor blk.26.ffn_norm.weight
create_tensor: loading tensor blk.26.ffn_gate.weight
create_tensor: loading tensor blk.26.ffn_down.weight
create_tensor: loading tensor blk.26.ffn_up.weight
create_tensor: loading tensor blk.27.attn_norm.weight
create_tensor: loading tensor blk.27.attn_q.weight
create_tensor: loading tensor blk.27.attn_k.weight
create_tensor: loading tensor blk.27.attn_v.weight
create_tensor: loading tensor blk.27.attn_output.weight
create_tensor: loading tensor blk.27.ffn_norm.weight
create_tensor: loading tensor blk.27.ffn_gate.weight
create_tensor: loading tensor blk.27.ffn_down.weight
create_tensor: loading tensor blk.27.ffn_up.weight
load_tensors: tensor 'token_embd.weight' (q6_K) (and 0 others) cannot be used with preferred buffer type CPU_REPACK, using CPU instead
-> Momentum Chrome ready — internal headless relay, clean profile, zero automation tells.
-> [SkillVault] Building Aho-Corasick <10ms intent index...
-> [SkillVault] Index ready: 9 skills loaded. LLM-driven selection active.
-> [Brain] Memory loaded: 4 domains, 4 cached selectors
-> Human Behavioral OS loaded.
Momentum backend live on 127.0.0.1:3000
ggml_metal_log_allocated_size: allocated buffer, size =  2207.12 MiB, ( 2207.50 / 10922.67)
load_tensors: offloading output layer to GPU
load_tensors: offloading 27 repeating layers to GPU
load_tensors: offloaded 29/29 layers to GPU
load_tensors:   CPU_Mapped model buffer size =   308.23 MiB
load_tensors:  MTL0_Mapped model buffer size =  2207.10 MiB
..............................................................................
llama_context: constructing llama_context
llama_context: n_seq_max     = 1
llama_context: n_ctx         = 8192
llama_context: n_ctx_seq     = 8192
llama_context: n_batch       = 2048
llama_context: n_ubatch      = 512
llama_context: causal_attn   = 1
llama_context: flash_attn    = enabled
llama_context: kv_unified    = false
llama_context: freq_base     = 500000.0
llama_context: freq_scale    = 1
llama_context: n_ctx_seq (8192) < n_ctx_train (131072) -- the full capacity of the model will not be utilized
ggml_metal_init: allocating
ggml_metal_init: found device: Apple M1 Pro
ggml_metal_init: picking default device: Apple M1 Pro
ggml_metal_init: use fusion         = true
ggml_metal_init: use concurrency    = true
ggml_metal_init: use graph optimize = true
set_abort_callback: call
llama_context:        CPU  output buffer size =     0.49 MiB
llama_kv_cache: layer   0: dev = MTL0
llama_kv_cache: layer   1: dev = MTL0
llama_kv_cache: layer   2: dev = MTL0
llama_kv_cache: layer   3: dev = MTL0
llama_kv_cache: layer   4: dev = MTL0
llama_kv_cache: layer   5: dev = MTL0
llama_kv_cache: layer   6: dev = MTL0
llama_kv_cache: layer   7: dev = MTL0
llama_kv_cache: layer   8: dev = MTL0
llama_kv_cache: layer   9: dev = MTL0
llama_kv_cache: layer  10: dev = MTL0
llama_kv_cache: layer  11: dev = MTL0
llama_kv_cache: layer  12: dev = MTL0
llama_kv_cache: layer  13: dev = MTL0
llama_kv_cache: layer  14: dev = MTL0
llama_kv_cache: layer  15: dev = MTL0
llama_kv_cache: layer  16: dev = MTL0
llama_kv_cache: layer  17: dev = MTL0
llama_kv_cache: layer  18: dev = MTL0
llama_kv_cache: layer  19: dev = MTL0
llama_kv_cache: layer  20: dev = MTL0
llama_kv_cache: layer  21: dev = MTL0
llama_kv_cache: layer  22: dev = MTL0
llama_kv_cache: layer  23: dev = MTL0
llama_kv_cache: layer  24: dev = MTL0
llama_kv_cache: layer  25: dev = MTL0
llama_kv_cache: layer  26: dev = MTL0
llama_kv_cache: layer  27: dev = MTL0
llama_kv_cache:       MTL0 KV buffer size =   896.00 MiB
llama_kv_cache: size =  896.00 MiB (  8192 cells,  28 layers,  1/1 seqs), K (f16):  448.00 MiB, V (f16):  448.00 MiB
llama_context: enumerating backends
llama_context: backend_ptrs.size() = 2
sched_reserve: reserving ...
sched_reserve: max_nodes = 2048
sched_reserve: reserving full memory module
sched_reserve: worst-case: n_tokens = 512, n_seqs = 1, n_outputs = 1
graph_reserve: reserving a graph for ubatch with n_tokens =    1, n_seqs =  1, n_outputs =    1
graph_reserve: reserving a graph for ubatch with n_tokens =  512, n_seqs =  1, n_outputs =  512
graph_reserve: reserving a graph for ubatch with n_tokens =    1, n_seqs =  1, n_outputs =    1
graph_reserve: reserving a graph for ubatch with n_tokens =  512, n_seqs =  1, n_outputs =  512
sched_reserve:       MTL0 compute buffer size =   256.50 MiB
sched_reserve:        CPU compute buffer size =    28.01 MiB
sched_reserve: graph nodes  = 875
sched_reserve: graph splits = 2
sched_reserve: reserve took 4.07 ms, sched copies = 1
Loading Momentum Vision Engine 0.5B...
llama_model_load_from_file_impl: using device MTL0 (Apple M1 Pro) (unknown id) - 7562 MiB free
llama_model_loader: loaded meta data with 26 key-value pairs and 291 tensors from /Users/sanjeevn/Downloads/Momentum AI/momentum-vision/momentum-vision-engine.gguf (version GGUF V3 (latest))
llama_model_loader: Dumping metadata keys/values. Note: KV overrides do not apply in this output.
llama_model_loader: - kv   0:                       general.architecture str              = qwen2
llama_model_loader: - kv   1:                               general.type str              = model
llama_model_loader: - kv   2:                               general.name str              = qwen2.5-0.5b-instruct
llama_model_loader: - kv   3:                            general.version str              = v0.1
llama_model_loader: - kv   4:                           general.finetune str              = qwen2.5-0.5b-instruct
llama_model_loader: - kv   5:                         general.size_label str              = 630M
llama_model_loader: - kv   6:                          qwen2.block_count u32              = 24
llama_model_loader: - kv   7:                       qwen2.context_length u32              = 32768
llama_model_loader: - kv   8:                     qwen2.embedding_length u32              = 896
llama_model_loader: - kv   9:                  qwen2.feed_forward_length u32              = 4864
llama_model_loader: - kv  10:                 qwen2.attention.head_count u32              = 14
llama_model_loader: - kv  11:              qwen2.attention.head_count_kv u32              = 2
llama_model_loader: - kv  12:                       qwen2.rope.freq_base f32              = 1000000.000000
llama_model_loader: - kv  13:     qwen2.attention.layer_norm_rms_epsilon f32              = 0.000001
llama_model_loader: - kv  14:                          general.file_type u32              = 15
llama_model_loader: - kv  15:                       tokenizer.ggml.model str              = gpt2
llama_model_loader: - kv  16:                         tokenizer.ggml.pre str              = qwen2
llama_model_loader: - kv  17:                      tokenizer.ggml.tokens arr[str,151936]  = ["!", "\"", "#", "$", "%", "&", "'", ...
llama_model_loader: - kv  18:                  tokenizer.ggml.token_type arr[i32,151936]  = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, ...
llama_model_loader: - kv  19:                      tokenizer.ggml.merges arr[str,151387]  = ["Ġ Ġ", "ĠĠ ĠĠ", "i n", "Ġ t",...
llama_model_loader: - kv  20:                tokenizer.ggml.eos_token_id u32              = 151645
llama_model_loader: - kv  21:            tokenizer.ggml.padding_token_id u32              = 151643
llama_model_loader: - kv  22:                tokenizer.ggml.bos_token_id u32              = 151643
llama_model_loader: - kv  23:               tokenizer.ggml.add_bos_token bool             = false
llama_model_loader: - kv  24:                    tokenizer.chat_template str              = {%- if tools %}\n    {{- '<|im_start|>...
llama_model_loader: - kv  25:               general.quantization_version u32              = 2
llama_model_loader: - type  f32:  121 tensors
llama_model_loader: - type q5_0:  133 tensors
llama_model_loader: - type q8_0:   13 tensors
llama_model_loader: - type q4_K:   12 tensors
llama_model_loader: - type q6_K:   12 tensors
print_info: file format = GGUF V3 (latest)
print_info: file type   = Q4_K - Medium
print_info: file size   = 462.96 MiB (6.16 BPW) 
init_tokenizer: initializing tokenizer for type 2
load: 0 unused tokens
load: control token: 151659 '<|fim_prefix|>' is not marked as EOG
load: control token: 151656 '<|video_pad|>' is not marked as EOG
load: control token: 151655 '<|image_pad|>' is not marked as EOG
load: control token: 151653 '<|vision_end|>' is not marked as EOG
load: control token: 151652 '<|vision_start|>' is not marked as EOG
load: control token: 151651 '<|quad_end|>' is not marked as EOG
load: control token: 151649 '<|box_end|>' is not marked as EOG
load: control token: 151648 '<|box_start|>' is not marked as EOG
load: control token: 151646 '<|object_ref_start|>' is not marked as EOG
load: control token: 151644 '<|im_start|>' is not marked as EOG
load: control token: 151661 '<|fim_suffix|>' is not marked as EOG
load: control token: 151647 '<|object_ref_end|>' is not marked as EOG
load: control-looking token: 128247 '</s>' was not control-type; this is probably a bug in the model. its type will be overridden
load: control token: 151660 '<|fim_middle|>' is not marked as EOG
load: control token: 151654 '<|vision_pad|>' is not marked as EOG
load: control token: 151650 '<|quad_start|>' is not marked as EOG
load: printing all EOG tokens:
load:   - 128247 ('</s>')
load:   - 151643 ('<|endoftext|>')
load:   - 151645 ('<|im_end|>')
load:   - 151662 ('<|fim_pad|>')
load:   - 151663 ('<|repo_name|>')
load:   - 151664 ('<|file_sep|>')
load: special tokens cache size = 23
load: token to piece cache size = 0.9310 MB
print_info: arch                  = qwen2
print_info: vocab_only            = 0
print_info: no_alloc              = 0
print_info: n_ctx_train           = 32768
print_info: n_embd                = 896
print_info: n_embd_inp            = 896
print_info: n_layer               = 24
print_info: n_head                = 14
print_info: n_head_kv             = 2
print_info: n_rot                 = 64
print_info: n_swa                 = 0
print_info: is_swa_any            = 0
print_info: n_embd_head_k         = 64
print_info: n_embd_head_v         = 64
print_info: n_gqa                 = 7
print_info: n_embd_k_gqa          = 128
print_info: n_embd_v_gqa          = 128
print_info: f_norm_eps            = 0.0e+00
print_info: f_norm_rms_eps        = 1.0e-06
print_info: f_clamp_kqv           = 0.0e+00
print_info: f_max_alibi_bias      = 0.0e+00
print_info: f_logit_scale         = 0.0e+00
print_info: f_attn_scale          = 0.0e+00
print_info: n_ff                  = 4864
print_info: n_expert              = 0
print_info: n_expert_used         = 0
print_info: n_expert_groups       = 0
print_info: n_group_used          = 0
print_info: causal attn           = 1
print_info: pooling type          = -1
print_info: rope type             = 2
print_info: rope scaling          = linear
print_info: freq_base_train       = 1000000.0
print_info: freq_scale_train      = 1
print_info: n_ctx_orig_yarn       = 32768
print_info: rope_yarn_log_mul     = 0.0000
print_info: rope_finetuned        = unknown
print_info: model type            = 1B
print_info: model params          = 630.17 M
print_info: general.name          = qwen2.5-0.5b-instruct
print_info: vocab type            = BPE
print_info: n_vocab               = 151936
print_info: n_merges              = 151387
print_info: BOS token             = 151643 '<|endoftext|>'
print_info: EOS token             = 151645 '<|im_end|>'
print_info: EOT token             = 151645 '<|im_end|>'
print_info: PAD token             = 151643 '<|endoftext|>'
print_info: LF token              = 198 'Ċ'
print_info: FIM PRE token         = 151659 '<|fim_prefix|>'
print_info: FIM SUF token         = 151661 '<|fim_suffix|>'
print_info: FIM MID token         = 151660 '<|fim_middle|>'
print_info: FIM PAD token         = 151662 '<|fim_pad|>'
print_info: FIM REP token         = 151663 '<|repo_name|>'
print_info: FIM SEP token         = 151664 '<|file_sep|>'
print_info: EOG token             = 128247 '</s>'
print_info: EOG token             = 151643 '<|endoftext|>'
print_info: EOG token             = 151645 '<|im_end|>'
print_info: EOG token             = 151662 '<|fim_pad|>'
print_info: EOG token             = 151663 '<|repo_name|>'
print_info: EOG token             = 151664 '<|file_sep|>'
print_info: max token length      = 256
load_tensors: loading model tensors, this can take a while... (mmap = true, direct_io = false)
load_tensors: layer   0 assigned to device MTL0, is_swa = 0
load_tensors: layer   1 assigned to device MTL0, is_swa = 0
load_tensors: layer   2 assigned to device MTL0, is_swa = 0
load_tensors: layer   3 assigned to device MTL0, is_swa = 0
load_tensors: layer   4 assigned to device MTL0, is_swa = 0
load_tensors: layer   5 assigned to device MTL0, is_swa = 0
load_tensors: layer   6 assigned to device MTL0, is_swa = 0
load_tensors: layer   7 assigned to device MTL0, is_swa = 0
load_tensors: layer   8 assigned to device MTL0, is_swa = 0
load_tensors: layer   9 assigned to device MTL0, is_swa = 0
load_tensors: layer  10 assigned to device MTL0, is_swa = 0
load_tensors: layer  11 assigned to device MTL0, is_swa = 0
load_tensors: layer  12 assigned to device MTL0, is_swa = 0
load_tensors: layer  13 assigned to device MTL0, is_swa = 0
load_tensors: layer  14 assigned to device MTL0, is_swa = 0
load_tensors: layer  15 assigned to device MTL0, is_swa = 0
load_tensors: layer  16 assigned to device MTL0, is_swa = 0
load_tensors: layer  17 assigned to device MTL0, is_swa = 0
load_tensors: layer  18 assigned to device MTL0, is_swa = 0
load_tensors: layer  19 assigned to device MTL0, is_swa = 0
load_tensors: layer  20 assigned to device MTL0, is_swa = 0
load_tensors: layer  21 assigned to device MTL0, is_swa = 0
load_tensors: layer  22 assigned to device MTL0, is_swa = 0
load_tensors: layer  23 assigned to device MTL0, is_swa = 0
load_tensors: layer  24 assigned to device MTL0, is_swa = 0
create_tensor: loading tensor token_embd.weight
create_tensor: loading tensor output_norm.weight
create_tensor: loading tensor output.weight
create_tensor: loading tensor blk.0.attn_norm.weight
create_tensor: loading tensor blk.0.attn_q.weight
create_tensor: loading tensor blk.0.attn_k.weight
create_tensor: loading tensor blk.0.attn_v.weight
create_tensor: loading tensor blk.0.attn_output.weight
create_tensor: loading tensor blk.0.attn_q.bias
create_tensor: loading tensor blk.0.attn_k.bias
create_tensor: loading tensor blk.0.attn_v.bias
create_tensor: loading tensor blk.0.ffn_norm.weight
create_tensor: loading tensor blk.0.ffn_gate.weight
create_tensor: loading tensor blk.0.ffn_down.weight
create_tensor: loading tensor blk.0.ffn_up.weight
create_tensor: loading tensor blk.1.attn_norm.weight
create_tensor: loading tensor blk.1.attn_q.weight
create_tensor: loading tensor blk.1.attn_k.weight
create_tensor: loading tensor blk.1.attn_v.weight
create_tensor: loading tensor blk.1.attn_output.weight
create_tensor: loading tensor blk.1.attn_q.bias
create_tensor: loading tensor blk.1.attn_k.bias
create_tensor: loading tensor blk.1.attn_v.bias
create_tensor: loading tensor blk.1.ffn_norm.weight
create_tensor: loading tensor blk.1.ffn_gate.weight
create_tensor: loading tensor blk.1.ffn_down.weight
create_tensor: loading tensor blk.1.ffn_up.weight
create_tensor: loading tensor blk.2.attn_norm.weight
create_tensor: loading tensor blk.2.attn_q.weight
create_tensor: loading tensor blk.2.attn_k.weight
create_tensor: loading tensor blk.2.attn_v.weight
create_tensor: loading tensor blk.2.attn_output.weight
create_tensor: loading tensor blk.2.attn_q.bias
create_tensor: loading tensor blk.2.attn_k.bias
create_tensor: loading tensor blk.2.attn_v.bias
create_tensor: loading tensor blk.2.ffn_norm.weight
create_tensor: loading tensor blk.2.ffn_gate.weight
create_tensor: loading tensor blk.2.ffn_down.weight
create_tensor: loading tensor blk.2.ffn_up.weight
create_tensor: loading tensor blk.3.attn_norm.weight
create_tensor: loading tensor blk.3.attn_q.weight
create_tensor: loading tensor blk.3.attn_k.weight
create_tensor: loading tensor blk.3.attn_v.weight
create_tensor: loading tensor blk.3.attn_output.weight
create_tensor: loading tensor blk.3.attn_q.bias
create_tensor: loading tensor blk.3.attn_k.bias
create_tensor: loading tensor blk.3.attn_v.bias
create_tensor: loading tensor blk.3.ffn_norm.weight
create_tensor: loading tensor blk.3.ffn_gate.weight
create_tensor: loading tensor blk.3.ffn_down.weight
create_tensor: loading tensor blk.3.ffn_up.weight
create_tensor: loading tensor blk.4.attn_norm.weight
create_tensor: loading tensor blk.4.attn_q.weight
create_tensor: loading tensor blk.4.attn_k.weight
create_tensor: loading tensor blk.4.attn_v.weight
create_tensor: loading tensor blk.4.attn_output.weight
create_tensor: loading tensor blk.4.attn_q.bias
create_tensor: loading tensor blk.4.attn_k.bias
create_tensor: loading tensor blk.4.attn_v.bias
create_tensor: loading tensor blk.4.ffn_norm.weight
create_tensor: loading tensor blk.4.ffn_gate.weight
create_tensor: loading tensor blk.4.ffn_down.weight
create_tensor: loading tensor blk.4.ffn_up.weight
create_tensor: loading tensor blk.5.attn_norm.weight
create_tensor: loading tensor blk.5.attn_q.weight
create_tensor: loading tensor blk.5.attn_k.weight
create_tensor: loading tensor blk.5.attn_v.weight
create_tensor: loading tensor blk.5.attn_output.weight
create_tensor: loading tensor blk.5.attn_q.bias
create_tensor: loading tensor blk.5.attn_k.bias
create_tensor: loading tensor blk.5.attn_v.bias
create_tensor: loading tensor blk.5.ffn_norm.weight
create_tensor: loading tensor blk.5.ffn_gate.weight
create_tensor: loading tensor blk.5.ffn_down.weight
create_tensor: loading tensor blk.5.ffn_up.weight
create_tensor: loading tensor blk.6.attn_norm.weight
create_tensor: loading tensor blk.6.attn_q.weight
create_tensor: loading tensor blk.6.attn_k.weight
create_tensor: loading tensor blk.6.attn_v.weight
create_tensor: loading tensor blk.6.attn_output.weight
create_tensor: loading tensor blk.6.attn_q.bias
create_tensor: loading tensor blk.6.attn_k.bias
create_tensor: loading tensor blk.6.attn_v.bias
create_tensor: loading tensor blk.6.ffn_norm.weight
create_tensor: loading tensor blk.6.ffn_gate.weight
create_tensor: loading tensor blk.6.ffn_down.weight
create_tensor: loading tensor blk.6.ffn_up.weight
create_tensor: loading tensor blk.7.attn_norm.weight
create_tensor: loading tensor blk.7.attn_q.weight
create_tensor: loading tensor blk.7.attn_k.weight
create_tensor: loading tensor blk.7.attn_v.weight
create_tensor: loading tensor blk.7.attn_output.weight
create_tensor: loading tensor blk.7.attn_q.bias
create_tensor: loading tensor blk.7.attn_k.bias
create_tensor: loading tensor blk.7.attn_v.bias
create_tensor: loading tensor blk.7.ffn_norm.weight
create_tensor: loading tensor blk.7.ffn_gate.weight
create_tensor: loading tensor blk.7.ffn_down.weight
create_tensor: loading tensor blk.7.ffn_up.weight
create_tensor: loading tensor blk.8.attn_norm.weight
create_tensor: loading tensor blk.8.attn_q.weight
create_tensor: loading tensor blk.8.attn_k.weight
create_tensor: loading tensor blk.8.attn_v.weight
create_tensor: loading tensor blk.8.attn_output.weight
create_tensor: loading tensor blk.8.attn_q.bias
create_tensor: loading tensor blk.8.attn_k.bias
create_tensor: loading tensor blk.8.attn_v.bias
create_tensor: loading tensor blk.8.ffn_norm.weight
create_tensor: loading tensor blk.8.ffn_gate.weight
create_tensor: loading tensor blk.8.ffn_down.weight
create_tensor: loading tensor blk.8.ffn_up.weight
create_tensor: loading tensor blk.9.attn_norm.weight
create_tensor: loading tensor blk.9.attn_q.weight
create_tensor: loading tensor blk.9.attn_k.weight
create_tensor: loading tensor blk.9.attn_v.weight
create_tensor: loading tensor blk.9.attn_output.weight
create_tensor: loading tensor blk.9.attn_q.bias
create_tensor: loading tensor blk.9.attn_k.bias
create_tensor: loading tensor blk.9.attn_v.bias
create_tensor: loading tensor blk.9.ffn_norm.weight
create_tensor: loading tensor blk.9.ffn_gate.weight
create_tensor: loading tensor blk.9.ffn_down.weight
create_tensor: loading tensor blk.9.ffn_up.weight
create_tensor: loading tensor blk.10.attn_norm.weight
create_tensor: loading tensor blk.10.attn_q.weight
create_tensor: loading tensor blk.10.attn_k.weight
create_tensor: loading tensor blk.10.attn_v.weight
create_tensor: loading tensor blk.10.attn_output.weight
create_tensor: loading tensor blk.10.attn_q.bias
create_tensor: loading tensor blk.10.attn_k.bias
create_tensor: loading tensor blk.10.attn_v.bias
create_tensor: loading tensor blk.10.ffn_norm.weight
create_tensor: loading tensor blk.10.ffn_gate.weight
create_tensor: loading tensor blk.10.ffn_down.weight
create_tensor: loading tensor blk.10.ffn_up.weight
create_tensor: loading tensor blk.11.attn_norm.weight
create_tensor: loading tensor blk.11.attn_q.weight
create_tensor: loading tensor blk.11.attn_k.weight
create_tensor: loading tensor blk.11.attn_v.weight
create_tensor: loading tensor blk.11.attn_output.weight
create_tensor: loading tensor blk.11.attn_q.bias
create_tensor: loading tensor blk.11.attn_k.bias
create_tensor: loading tensor blk.11.attn_v.bias
create_tensor: loading tensor blk.11.ffn_norm.weight
create_tensor: loading tensor blk.11.ffn_gate.weight
create_tensor: loading tensor blk.11.ffn_down.weight
create_tensor: loading tensor blk.11.ffn_up.weight
create_tensor: loading tensor blk.12.attn_norm.weight
create_tensor: loading tensor blk.12.attn_q.weight
create_tensor: loading tensor blk.12.attn_k.weight
create_tensor: loading tensor blk.12.attn_v.weight
create_tensor: loading tensor blk.12.attn_output.weight
create_tensor: loading tensor blk.12.attn_q.bias
create_tensor: loading tensor blk.12.attn_k.bias
create_tensor: loading tensor blk.12.attn_v.bias
create_tensor: loading tensor blk.12.ffn_norm.weight
create_tensor: loading tensor blk.12.ffn_gate.weight
create_tensor: loading tensor blk.12.ffn_down.weight
create_tensor: loading tensor blk.12.ffn_up.weight
create_tensor: loading tensor blk.13.attn_norm.weight
create_tensor: loading tensor blk.13.attn_q.weight
create_tensor: loading tensor blk.13.attn_k.weight
create_tensor: loading tensor blk.13.attn_v.weight
create_tensor: loading tensor blk.13.attn_output.weight
create_tensor: loading tensor blk.13.attn_q.bias
create_tensor: loading tensor blk.13.attn_k.bias
create_tensor: loading tensor blk.13.attn_v.bias
create_tensor: loading tensor blk.13.ffn_norm.weight
create_tensor: loading tensor blk.13.ffn_gate.weight
create_tensor: loading tensor blk.13.ffn_down.weight
create_tensor: loading tensor blk.13.ffn_up.weight
create_tensor: loading tensor blk.14.attn_norm.weight
create_tensor: loading tensor blk.14.attn_q.weight
create_tensor: loading tensor blk.14.attn_k.weight
create_tensor: loading tensor blk.14.attn_v.weight
create_tensor: loading tensor blk.14.attn_output.weight
create_tensor: loading tensor blk.14.attn_q.bias
create_tensor: loading tensor blk.14.attn_k.bias
create_tensor: loading tensor blk.14.attn_v.bias
create_tensor: loading tensor blk.14.ffn_norm.weight
create_tensor: loading tensor blk.14.ffn_gate.weight
create_tensor: loading tensor blk.14.ffn_down.weight
create_tensor: loading tensor blk.14.ffn_up.weight
create_tensor: loading tensor blk.15.attn_norm.weight
create_tensor: loading tensor blk.15.attn_q.weight
create_tensor: loading tensor blk.15.attn_k.weight
create_tensor: loading tensor blk.15.attn_v.weight
create_tensor: loading tensor blk.15.attn_output.weight
create_tensor: loading tensor blk.15.attn_q.bias
create_tensor: loading tensor blk.15.attn_k.bias
create_tensor: loading tensor blk.15.attn_v.bias
create_tensor: loading tensor blk.15.ffn_norm.weight
create_tensor: loading tensor blk.15.ffn_gate.weight
create_tensor: loading tensor blk.15.ffn_down.weight
create_tensor: loading tensor blk.15.ffn_up.weight
create_tensor: loading tensor blk.16.attn_norm.weight
create_tensor: loading tensor blk.16.attn_q.weight
create_tensor: loading tensor blk.16.attn_k.weight
create_tensor: loading tensor blk.16.attn_v.weight
create_tensor: loading tensor blk.16.attn_output.weight
create_tensor: loading tensor blk.16.attn_q.bias
create_tensor: loading tensor blk.16.attn_k.bias
create_tensor: loading tensor blk.16.attn_v.bias
create_tensor: loading tensor blk.16.ffn_norm.weight
create_tensor: loading tensor blk.16.ffn_gate.weight
create_tensor: loading tensor blk.16.ffn_down.weight
create_tensor: loading tensor blk.16.ffn_up.weight
create_tensor: loading tensor blk.17.attn_norm.weight
create_tensor: loading tensor blk.17.attn_q.weight
create_tensor: loading tensor blk.17.attn_k.weight
create_tensor: loading tensor blk.17.attn_v.weight
create_tensor: loading tensor blk.17.attn_output.weight
create_tensor: loading tensor blk.17.attn_q.bias
create_tensor: loading tensor blk.17.attn_k.bias
create_tensor: loading tensor blk.17.attn_v.bias
create_tensor: loading tensor blk.17.ffn_norm.weight
create_tensor: loading tensor blk.17.ffn_gate.weight
create_tensor: loading tensor blk.17.ffn_down.weight
create_tensor: loading tensor blk.17.ffn_up.weight
create_tensor: loading tensor blk.18.attn_norm.weight
create_tensor: loading tensor blk.18.attn_q.weight
create_tensor: loading tensor blk.18.attn_k.weight
create_tensor: loading tensor blk.18.attn_v.weight
create_tensor: loading tensor blk.18.attn_output.weight
create_tensor: loading tensor blk.18.attn_q.bias
create_tensor: loading tensor blk.18.attn_k.bias
create_tensor: loading tensor blk.18.attn_v.bias
create_tensor: loading tensor blk.18.ffn_norm.weight
create_tensor: loading tensor blk.18.ffn_gate.weight
create_tensor: loading tensor blk.18.ffn_down.weight
create_tensor: loading tensor blk.18.ffn_up.weight
create_tensor: loading tensor blk.19.attn_norm.weight
create_tensor: loading tensor blk.19.attn_q.weight
create_tensor: loading tensor blk.19.attn_k.weight
create_tensor: loading tensor blk.19.attn_v.weight
create_tensor: loading tensor blk.19.attn_output.weight
create_tensor: loading tensor blk.19.attn_q.bias
create_tensor: loading tensor blk.19.attn_k.bias
create_tensor: loading tensor blk.19.attn_v.bias
create_tensor: loading tensor blk.19.ffn_norm.weight
create_tensor: loading tensor blk.19.ffn_gate.weight
create_tensor: loading tensor blk.19.ffn_down.weight
create_tensor: loading tensor blk.19.ffn_up.weight
create_tensor: loading tensor blk.20.attn_norm.weight
create_tensor: loading tensor blk.20.attn_q.weight
create_tensor: loading tensor blk.20.attn_k.weight
create_tensor: loading tensor blk.20.attn_v.weight
create_tensor: loading tensor blk.20.attn_output.weight
create_tensor: loading tensor blk.20.attn_q.bias
create_tensor: loading tensor blk.20.attn_k.bias
create_tensor: loading tensor blk.20.attn_v.bias
create_tensor: loading tensor blk.20.ffn_norm.weight
create_tensor: loading tensor blk.20.ffn_gate.weight
create_tensor: loading tensor blk.20.ffn_down.weight
create_tensor: loading tensor blk.20.ffn_up.weight
create_tensor: loading tensor blk.21.attn_norm.weight
create_tensor: loading tensor blk.21.attn_q.weight
create_tensor: loading tensor blk.21.attn_k.weight
create_tensor: loading tensor blk.21.attn_v.weight
create_tensor: loading tensor blk.21.attn_output.weight
create_tensor: loading tensor blk.21.attn_q.bias
create_tensor: loading tensor blk.21.attn_k.bias
create_tensor: loading tensor blk.21.attn_v.bias
create_tensor: loading tensor blk.21.ffn_norm.weight
create_tensor: loading tensor blk.21.ffn_gate.weight
create_tensor: loading tensor blk.21.ffn_down.weight
create_tensor: loading tensor blk.21.ffn_up.weight
create_tensor: loading tensor blk.22.attn_norm.weight
create_tensor: loading tensor blk.22.attn_q.weight
create_tensor: loading tensor blk.22.attn_k.weight
create_tensor: loading tensor blk.22.attn_v.weight
create_tensor: loading tensor blk.22.attn_output.weight
create_tensor: loading tensor blk.22.attn_q.bias
create_tensor: loading tensor blk.22.attn_k.bias
create_tensor: loading tensor blk.22.attn_v.bias
create_tensor: loading tensor blk.22.ffn_norm.weight
create_tensor: loading tensor blk.22.ffn_gate.weight
create_tensor: loading tensor blk.22.ffn_down.weight
create_tensor: loading tensor blk.22.ffn_up.weight
create_tensor: loading tensor blk.23.attn_norm.weight
create_tensor: loading tensor blk.23.attn_q.weight
create_tensor: loading tensor blk.23.attn_k.weight
create_tensor: loading tensor blk.23.attn_v.weight
create_tensor: loading tensor blk.23.attn_output.weight
create_tensor: loading tensor blk.23.attn_q.bias
create_tensor: loading tensor blk.23.attn_k.bias
create_tensor: loading tensor blk.23.attn_v.bias
create_tensor: loading tensor blk.23.ffn_norm.weight
create_tensor: loading tensor blk.23.ffn_gate.weight
create_tensor: loading tensor blk.23.ffn_down.weight
create_tensor: loading tensor blk.23.ffn_up.weight
load_tensors: tensor 'token_embd.weight' (q5_0) (and 0 others) cannot be used with preferred buffer type CPU_REPACK, using CPU instead
ggml_metal_log_allocated_size: allocated buffer, size =   462.97 MiB, ( 3822.97 / 10922.67)
load_tensors: offloading output layer to GPU
load_tensors: offloading 23 repeating layers to GPU
load_tensors: offloaded 25/25 layers to GPU
load_tensors:   CPU_Mapped model buffer size =    89.26 MiB
load_tensors:  MTL0_Mapped model buffer size =   462.96 MiB
.....................................................
llama_context: constructing llama_context
llama_context: n_seq_max     = 1
llama_context: n_ctx         = 2048
llama_context: n_ctx_seq     = 2048
llama_context: n_batch       = 2048
llama_context: n_ubatch      = 512
llama_context: causal_attn   = 1
llama_context: flash_attn    = enabled
llama_context: kv_unified    = false
llama_context: freq_base     = 1000000.0
llama_context: freq_scale    = 1
llama_context: n_ctx_seq (2048) < n_ctx_train (32768) -- the full capacity of the model will not be utilized
ggml_metal_init: allocating
ggml_metal_init: found device: Apple M1 Pro
ggml_metal_init: picking default device: Apple M1 Pro
ggml_metal_init: use fusion         = true
ggml_metal_init: use concurrency    = true
ggml_metal_init: use graph optimize = true
set_abort_callback: call
llama_context:        CPU  output buffer size =     0.58 MiB
llama_kv_cache: layer   0: dev = MTL0
llama_kv_cache: layer   1: dev = MTL0
llama_kv_cache: layer   2: dev = MTL0
llama_kv_cache: layer   3: dev = MTL0
llama_kv_cache: layer   4: dev = MTL0
llama_kv_cache: layer   5: dev = MTL0
llama_kv_cache: layer   6: dev = MTL0
llama_kv_cache: layer   7: dev = MTL0
llama_kv_cache: layer   8: dev = MTL0
llama_kv_cache: layer   9: dev = MTL0
llama_kv_cache: layer  10: dev = MTL0
llama_kv_cache: layer  11: dev = MTL0
llama_kv_cache: layer  12: dev = MTL0
llama_kv_cache: layer  13: dev = MTL0
llama_kv_cache: layer  14: dev = MTL0
llama_kv_cache: layer  15: dev = MTL0
llama_kv_cache: layer  16: dev = MTL0
llama_kv_cache: layer  17: dev = MTL0
llama_kv_cache: layer  18: dev = MTL0
llama_kv_cache: layer  19: dev = MTL0
llama_kv_cache: layer  20: dev = MTL0
llama_kv_cache: layer  21: dev = MTL0
llama_kv_cache: layer  22: dev = MTL0
llama_kv_cache: layer  23: dev = MTL0
llama_kv_cache:       MTL0 KV buffer size =    24.00 MiB
llama_kv_cache: size =   24.00 MiB (  2048 cells,  24 layers,  1/1 seqs), K (f16):   12.00 MiB, V (f16):   12.00 MiB
llama_context: enumerating backends
llama_context: backend_ptrs.size() = 2
sched_reserve: reserving ...
sched_reserve: max_nodes = 2328
sched_reserve: reserving full memory module
sched_reserve: worst-case: n_tokens = 512, n_seqs = 1, n_outputs = 1
graph_reserve: reserving a graph for ubatch with n_tokens =    1, n_seqs =  1, n_outputs =    1
graph_reserve: reserving a graph for ubatch with n_tokens =  512, n_seqs =  1, n_outputs =  512
graph_reserve: reserving a graph for ubatch with n_tokens =    1, n_seqs =  1, n_outputs =    1
graph_reserve: reserving a graph for ubatch with n_tokens =  512, n_seqs =  1, n_outputs =  512
sched_reserve:       MTL0 compute buffer size =   298.50 MiB
sched_reserve:        CPU compute buffer size =     7.51 MiB
sched_reserve: graph nodes  = 823
sched_reserve: graph splits = 2
sched_reserve: reserve took 4.66 ms, sched copies = 1
Dual-Cognition Models Mounted Successfully! Awaiting user input...
Frontend Screencast connected.
Frontend connected.
-> [Conversational] No browser capture (Chrome stays hidden).
Vision Engine: bypassed (preventing hallucination/latency until Qwen2-VL is integrated).
Engine prompt assembled: 3132 bytes
Engine prompt: 3132 bytes → 683 tokens (KV Cache Match: 0 tokens)
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_rms_norm_mul_f32_4', name = 'kernel_rms_norm_mul_f32_4'
ggml_metal_library_compile_pipeline: loaded kernel_rms_norm_mul_f32_4                     0x1222093d0 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_cpy_f32_f16', name = 'kernel_cpy_f32_f16'
ggml_metal_library_compile_pipeline: loaded kernel_cpy_f32_f16                            0x1223074b0 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_mul_mm_q5_K_f32', name = 'kernel_mul_mm_q5_K_f32_bci=0_bco=0'
ggml_metal_library_compile_pipeline: loaded kernel_mul_mm_q5_K_f32_bci=0_bco=0            0x122307f30 | th_max =  832 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_mul_mm_q6_K_f32', name = 'kernel_mul_mm_q6_K_f32_bci=0_bco=0'
ggml_metal_library_compile_pipeline: loaded kernel_mul_mm_q6_K_f32_bci=0_bco=0            0x122506830 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_rope_norm_f32', name = 'kernel_rope_norm_f32_imrope=0'
ggml_metal_library_compile_pipeline: loaded kernel_rope_norm_f32_imrope=0                 0x122209a30 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_set_rows_f16_i64', name = 'kernel_set_rows_f16_i64'
ggml_metal_library_compile_pipeline: loaded kernel_set_rows_f16_i64                       0x12220a160 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_flash_attn_ext_blk', name = 'kernel_flash_attn_ext_blk_nqptg=8_ncpsg=64'
ggml_metal_library_compile_pipeline: loaded kernel_flash_attn_ext_blk_nqptg=8_ncpsg=64      0x12220ab20 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_flash_attn_ext_f16_dk128_dv128', name = 'kernel_flash_attn_ext_f16_dk128_dv128_mask=1_sinks=0_bias=0_scap=0_kvpad=0_bcm=0_ns10=1024_ns20=1024_nsg=4'
ggml_metal_library_compile_pipeline: loaded kernel_flash_attn_ext_f16_dk128_dv128_mask=1_sinks=0_bias=0_scap=0_kvpad=0_bcm=0_ns10=1024_ns20=1024_nsg=4      0x12220ad80 | th_max =  768 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_bin_fuse_f32_f32_f32_4', name = 'kernel_bin_fuse_f32_f32_f32_4_op=0_nf=1_rb=0'
ggml_metal_library_compile_pipeline: loaded kernel_bin_fuse_f32_f32_f32_4_op=0_nf=1_rb=0      0x12220a690 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_swiglu_f32', name = 'kernel_swiglu_f32'
ggml_metal_library_compile_pipeline: loaded kernel_swiglu_f32                             0x12220b3b0 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_mul_mm_q5_K_f32', name = 'kernel_mul_mm_q5_K_f32_bci=0_bco=1'
ggml_metal_library_compile_pipeline: loaded kernel_mul_mm_q5_K_f32_bci=0_bco=1            0x105c06350 | th_max =  832 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_mul_mm_q6_K_f32', name = 'kernel_mul_mm_q6_K_f32_bci=0_bco=1'
ggml_metal_library_compile_pipeline: loaded kernel_mul_mm_q6_K_f32_bci=0_bco=1            0x105c067d0 | th_max =  896 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_flash_attn_ext_f16_dk128_dv128', name = 'kernel_flash_attn_ext_f16_dk128_dv128_mask=1_sinks=0_bias=0_scap=0_kvpad=0_bcm=1_ns10=1024_ns20=1024_nsg=4'
ggml_metal_library_compile_pipeline: loaded kernel_flash_attn_ext_f16_dk128_dv128_mask=1_sinks=0_bias=0_scap=0_kvpad=0_bcm=1_ns10=1024_ns20=1024_nsg=4      0x105c06e00 | th_max =  768 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_get_rows_f32', name = 'kernel_get_rows_f32'
ggml_metal_library_compile_pipeline: loaded kernel_get_rows_f32                           0x1223089d0 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_bin_fuse_f32_f32_f32_4', name = 'kernel_bin_fuse_f32_f32_f32_4_op=0_nf=1_rb=1'
ggml_metal_library_compile_pipeline: loaded kernel_bin_fuse_f32_f32_f32_4_op=0_nf=1_rb=1      0x122009fd0 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_mul_mv_q5_K_f32', name = 'kernel_mul_mv_q5_K_f32_nsg=2'
ggml_metal_library_compile_pipeline: loaded kernel_mul_mv_q5_K_f32_nsg=2                  0x1223097a0 | th_max =  576 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_mul_mv_q6_K_f32', name = 'kernel_mul_mv_q6_K_f32_nsg=2'
ggml_metal_library_compile_pipeline: loaded kernel_mul_mv_q6_K_f32_nsg=2                  0x122309d50 | th_max = 1024 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_flash_attn_ext_vec_f16_dk128_dv128', name = 'kernel_flash_attn_ext_vec_f16_dk128_dv128_mask=1_sink=0_bias=0_scap=0_kvpad=0_ns10=1024_ns20=1024_nsg=1_nwg=32'
ggml_metal_library_compile_pipeline: loaded kernel_flash_attn_ext_vec_f16_dk128_dv128_mask=1_sink=0_bias=0_scap=0_kvpad=0_ns10=1024_ns20=1024_nsg=1_nwg=32      0x12210af10 | th_max =  448 | th_width =   32
ggml_metal_library_compile_pipeline: compiling pipeline: base = 'kernel_flash_attn_ext_vec_reduce', name = 'kernel_flash_attn_ext_vec_reduce_dv=128_nwg=32'
ggml_metal_library_compile_pipeline: loaded kernel_flash_attn_ext_vec_reduce_dv=128_nwg=32      0x12210b9b0 | th_max = 1024 | th_width =   32
-> [Engine] Stopped: Hit EOS token
[FATAL] JSON schema violation — raw: {"thought":"Finally, a real conversation","action":{"type":"update_memory","key":"user_name","data":"Danny"},"action":{"type":"chat"},"speech":"Hi Danny, what's on your mind?
-> [Conversational] No browser capture (Chrome stays hidden).
Vision Engine: bypassed (preventing hallucination/latency until Qwen2-VL is integrated).
Engine prompt assembled: 3088 bytes
Engine prompt: 3088 bytes → 673 tokens (KV Cache Match: 574 tokens)
-> [Engine] Stopped: Hit EOS token
[EXEC] Action: UpdateMemory { key: "user_name", data: String("Danny") }
-> [Memory] Store: user_name
-> [Conversational] No browser capture (Chrome stays hidden).
Vision Engine: bypassed (preventing hallucination/latency until Qwen2-VL is integrated).
Engine prompt assembled: 3092 bytes
Engine prompt: 3092 bytes → 673 tokens (KV Cache Match: 554 tokens)
-> [Engine] Stopped: Hit EOS token
[EXEC] Action: Click { anchor: "https://www.indeed.com/", button: "" }
-> Click 'https://www.indeed.com/' ()
! Click failed: https://www.indeed.com/
Vision Engine: bypassed (preventing hallucination/latency until Qwen2-VL is integrated).
Engine prompt assembled: 2568 bytes
Engine prompt: 2568 bytes → 589 tokens (KV Cache Match: 552 tokens)
-> [Engine] Stopped: stream_tx channel closed

