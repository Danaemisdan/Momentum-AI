#!/usr/bin/env python3
"""
Momentum Knowledge Graph Bootstrap
────────────────────────────────────
Calls Gemini API with highly specific prompts to generate the initial
knowledge graph seed. Runs in multiple passes, each building on the last.
Output: ~/.momentum/graph.json

Usage:
    python3 bootstrap_graph.py --api-key YOUR_GEMINI_API_KEY
    python3 bootstrap_graph.py --api-key YOUR_KEY --passes industries platforms tactics sources
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
import urllib.request
import urllib.error

# ─────────────────────────────────────────────
# CONFIG
# ─────────────────────────────────────────────

GRAPH_PATH = Path.home() / ".momentum" / "graph.json"
GEMINI_URL = "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash-lite:generateContent"
NOW = datetime.now(timezone.utc).isoformat()

VALID_NODE_TYPES = [
    "person", "platform", "app_category", "industry",
    "domain", "tactic", "task", "concept", "company", "source"
]

VALID_EDGE_RELS = [
    "KNOWS", "REACHABLE_ON", "WORKS_AT",
    "HAS_CAPABILITY", "RUNS_ON", "BEST_FOR", "PART_OF", "REQUIRES",
    "USED_TACTIC", "INVOLVED", "SUCCEEDED_IN", "FAILED_IN", "LEARNED_FROM",
    "MORE_RELIABLE_THAN", "PRIMARY_SOURCE_FOR", "AGGREGATES_FROM", "RELATED_TO"
]


# ─────────────────────────────────────────────
# GEMINI CALLER
# ─────────────────────────────────────────────

def call_gemini(api_key: str, prompt: str, temperature: float = 0.2, retries: int = 3) -> str:
    payload = {
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {
            "temperature": temperature,
            "maxOutputTokens": 8192,
            # responseMimeType not supported on gemini-3.8-flash — we enforce JSON via prompt instead
        }
    }
    data = json.dumps(payload).encode("utf-8")
    url = f"{GEMINI_URL}?key={api_key}"
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"}, method="POST")

    for attempt in range(retries):
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                body = json.loads(resp.read().decode("utf-8"))
                return body["candidates"][0]["content"]["parts"][0]["text"]
        except urllib.error.HTTPError as e:
            err_body = e.read().decode()
            if e.code == 503:
                wait = 2 ** attempt * 5  # 5s, 10s, 20s
                print(f"  ⏳ 503 overload, retrying in {wait}s (attempt {attempt+1}/{retries})...", file=sys.stderr)
                time.sleep(wait)
                continue
            print(f"  ✗ Gemini API error {e.code}: {err_body}", file=sys.stderr)
            return ""
        except Exception as e:
            print(f"  ✗ Request failed: {e}", file=sys.stderr)
            if attempt < retries - 1:
                time.sleep(3)
                continue
            return ""
    return ""


# ─────────────────────────────────────────────
# GRAPH HELPERS
# ─────────────────────────────────────────────

def load_graph() -> dict:
    if GRAPH_PATH.exists():
        return json.loads(GRAPH_PATH.read_text())
    return {"version": "3.0.0", "nodes": {}, "edges": []}


def save_graph(graph: dict):
    GRAPH_PATH.parent.mkdir(parents=True, exist_ok=True)
    tmp = GRAPH_PATH.with_suffix(".tmp")
    tmp.write_text(json.dumps(graph, indent=2))
    tmp.rename(GRAPH_PATH)
    print(f"  ✓ Saved → {GRAPH_PATH} ({len(graph['nodes'])} nodes, {len(graph['edges'])} edges)")


def make_node(node_id: str, node_type: str, label: str, meta: dict) -> dict:
    return {
        "id": node_id,
        "node_type": node_type,
        "label": label,
        "meta": meta,
        "created_at": NOW,
        "updated_at": NOW
    }


def make_edge(from_id: str, to_id: str, rel: str, weight: float = 1.0, meta: dict = None) -> dict:
    return {
        "from": from_id,
        "to": to_id,
        "rel": rel,
        "weight": weight,
        "meta": meta or {},
        "created_at": NOW
    }


def merge_into_graph(graph: dict, new_nodes: list, new_edges: list):
    for node in new_nodes:
        nid = node.get("id", "")
        if not nid:
            continue
        if nid not in graph["nodes"]:
            graph["nodes"][nid] = node
        else:
            graph["nodes"][nid]["meta"].update(node.get("meta", {}))
            graph["nodes"][nid]["updated_at"] = NOW

    for edge in new_edges:
        if not edge.get("from") or not edge.get("to") or not edge.get("rel"):
            continue
        exists = any(
            e["from"] == edge["from"] and e["to"] == edge["to"] and e["rel"] == edge["rel"]
            for e in graph["edges"]
        )
        if not exists:
            graph["edges"].append(edge)


def parse_response(raw: str) -> tuple[list, list]:
    """Parse Gemini response into (nodes, edges). Handles pure JSON or markdown-wrapped JSON."""
    if not raw:
        return [], []

    # Try direct parse first
    try:
        data = json.loads(raw)
        return data.get("nodes", []), data.get("edges", [])
    except json.JSONDecodeError:
        pass

    # Try to extract from markdown code block (```json ... ``` or ``` ... ```)
    import re
    patterns = [
        r"```json\s*([\s\S]+?)```",
        r"```\s*([\s\S]+?)```",
        r"(\{[\s\S]+\})",  # raw JSON object anywhere in response
    ]
    for pattern in patterns:
        match = re.search(pattern, raw)
        if match:
            try:
                data = json.loads(match.group(1))
                return data.get("nodes", []), data.get("edges", [])
            except json.JSONDecodeError:
                continue

    print(f"  ✗ Could not parse JSON from response (first 200 chars): {raw[:200]}", file=sys.stderr)
    return [], []


# ─────────────────────────────────────────────
# PASS DEFINITIONS
# ─────────────────────────────────────────────

SCHEMA_REMINDER = """
IMPORTANT: Output ONLY valid JSON. No markdown. No backticks. No explanation before or after. Start your response with { and end with }.

Required format:
{
  "nodes": [
    {
      "id": "<type>:<snake_case_identifier>",
      "node_type": "<one of: """ + ", ".join(VALID_NODE_TYPES) + """>",
      "label": "<human readable name>",
      "meta": { <any relevant key-value pairs as strings or arrays> }
    }
  ],
  "edges": [
    {
      "from": "<node id>",
      "to": "<node id>",
      "rel": "<one of: """ + ", ".join(VALID_EDGE_RELS) + """>",
      "weight": <0.0 to 1.0>,
      "meta": {}
    }
  ]
}
"""


def gemini_batch(api_key: str, items: list[str], node_type: str, meta_instructions: str, edge_instructions: str = "") -> tuple[list, list]:
    """Ask Gemini to generate exactly these items (max 10 at a time) as graph nodes."""
    items_str = "\n".join(f"- {item}" for item in items)
    prompt = f"""You are seeding a knowledge graph for an AI automation agent.
Generate graph nodes for EXACTLY these {len(items)} items (no more, no less):
{items_str}

Node type for all: "{node_type}"

For each node, include these meta fields:
{meta_instructions}

{edge_instructions}

{SCHEMA_REMINDER}"""
    return parse_response(call_gemini(api_key, prompt))


def batched(lst: list, size: int):
    for i in range(0, len(lst), size):
        yield lst[i:i + size]


def pass_app_categories(api_key: str, graph: dict):
    print("\n[Pass 0] App categories...")
    categories = [
        "messaging", "browser", "email_client", "terminal", "code_editor",
        "file_manager", "media_player", "video_conferencing", "calendar",
        "notes", "task_manager", "design_tool", "3d_modeling", "spreadsheet",
        "document_editor", "photo_editor", "developer_tools", "database_client",
        "api_client", "social_media_app", "music_streaming", "video_streaming",
        "cloud_storage", "password_manager", "presentation", "pdf_reader",
        "screen_recorder", "browser_automation", "crm", "accounting"
    ]
    meta = """\
- "description": what apps in this category do
- "interaction_model": how an AI agent interacts with apps in this category (e.g. "has contact list and compose area", "has address bar and tabs")
- "common_examples": JSON array of 3-4 well-known app names in this category"""

    total_nodes, total_edges = [], []
    for batch in batched(categories, 10):
        nodes, edges = gemini_batch(api_key, batch, "app_category", meta)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} category nodes, {len(total_edges)} edges")


def pass_industries(api_key: str, graph: dict):
    print("\n[Pass 1] Industry taxonomy...")
    industries_batch1 = [
        "saas", "fintech", "healthtech", "edtech", "ecommerce", "real_estate",
        "manufacturing", "logistics", "consulting", "legal"
    ]
    industries_batch2 = [
        "healthcare", "construction", "retail", "hospitality", "media_entertainment",
        "gaming", "marketing_agency", "recruiting", "non_profit", "government"
    ]
    industries_batch3 = [
        "cybersecurity", "ai_ml", "biotech", "cleantech", "agriculture",
        "automotive", "aerospace", "insurance", "banking", "real_estate_tech"
    ]
    meta = """\
- "description": what this industry is and who the key players are (1-2 sentences)
- "b2b_or_b2c": "b2b", "b2c", or "both"
- "typical_goals": JSON array of 3-4 common business goals in this industry
- "key_job_titles": JSON array of key decision-maker roles"""

    total_nodes, total_edges = [], []
    for batch in [industries_batch1, industries_batch2, industries_batch3]:
        nodes, edges = gemini_batch(api_key, batch, "industry", meta)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} industry nodes, {len(total_edges)} edges")


def pass_platforms(api_key: str, graph: dict):
    print("\n[Pass 2] Platforms...")
    platforms_b1 = ["linkedin", "twitter_x", "youtube", "instagram", "facebook", "whatsapp", "telegram", "discord", "slack", "reddit"]
    platforms_b2 = ["github", "gitlab", "notion", "airtable", "hubspot", "salesforce", "gmail", "google_calendar", "google_docs", "google_sheets"]
    platforms_b3 = ["zoom", "google_meet", "figma", "canva", "shopify", "amazon", "producthunt", "crunchbase", "angellist", "glassdoor"]
    platforms_b4 = ["indeed", "trello", "asana", "jira", "confluence", "stripe", "twilio", "zapier", "make_integromat", "typeform"]

    meta = """\
- "platform_type": "web", "desktop_app", "mobile_app", "api", or "cli"
- "nature": 1-sentence description of what this platform is and who uses it
- "primary_use_cases": JSON array of 3-5 specific things an AI agent can do here
- "login_required": true or false
- "target": the main URL or app name to open"""

    edge_instructions = 'Also include BEST_FOR edges linking each platform to the most relevant industry IDs (format: "industry:saas", "industry:recruiting", etc.)'

    total_nodes, total_edges = [], []
    for batch in [platforms_b1, platforms_b2, platforms_b3, platforms_b4]:
        nodes, edges = gemini_batch(api_key, batch, "platform", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} platform nodes, {len(total_edges)} edges")


def pass_domains_and_tactics(api_key: str, graph: dict):
    print("\n[Pass 3] Domains + tactics...")

    # First generate domains
    domains = ["sales", "communication", "research", "coding", "content_creation", "management", "lead_gen"]
    domain_meta = """\
- "description": what this skill domain covers (1 sentence)
- "examples": JSON array of 3-4 concrete tasks an agent would do in this domain"""

    d_nodes, d_edges = gemini_batch(api_key, domains, "domain", domain_meta)
    merge_into_graph(graph, d_nodes, d_edges)
    time.sleep(1)

    # Then generate tactics per domain in batches
    tactics_map = {
        "sales": ["cold_outreach", "warm_outreach", "followup_email", "pitch_deck", "objection_handling", "closing"],
        "communication": ["message_drafting", "email_writing", "meeting_request", "negotiation", "thank_you_note"],
        "research": ["web_research", "competitor_analysis", "market_research", "person_lookup", "company_lookup"],
        "coding": ["bug_finding", "code_review", "feature_implementation", "test_writing", "refactoring", "documentation"],
        "content_creation": ["blog_writing", "social_post", "ad_copy", "seo_optimization", "script_writing"],
        "lead_gen": ["identify_leads", "qualify_leads", "build_prospect_list", "enrich_contact_data"],
    }

    tactic_meta = """\
- "description": exactly what this tactic does (1 sentence)
- "required_inputs": JSON array of input slots needed before execution (e.g. ["contact_name", "message_body"])
- "optional_inputs": JSON array of optional params (e.g. ["tone", "follow_up_days"])
- "success_signals": what observable outcome confirms success
- "best_context": what industry or situation this works best in"""

    total_nodes, total_edges = list(d_nodes), list(d_edges)
    for domain, tactics in tactics_map.items():
        edge_instructions = f'Include PART_OF edges from each tactic to "domain:{domain}". Include RUNS_ON edges to relevant platform IDs where applicable (e.g. "platform:linkedin", "platform:gmail").'
        nodes, edges = gemini_batch(api_key, tactics, "tactic", tactic_meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)

    print(f"  + {len(total_nodes)} domain/tactic nodes, {len(total_edges)} edges")


def pass_information_sources(api_key: str, graph: dict):
    print("\n[Pass 4] Information sources...")
    sources_b1 = ["reuters", "bbc", "techcrunch", "hacker_news", "reddit", "twitter_x_feed", "arxiv", "google_scholar", "github_trending", "producthunt_feed"]
    sources_b2 = ["crunchbase_data", "glassdoor_reviews", "trustpilot", "g2_reviews", "wikipedia", "company_official_blog", "sec_filings", "stack_overflow", "medium", "substack"]

    meta = """\
- "nature": type of source — "primary_outlet", "aggregator", "community", "official", "academic", "review_site", "social"
- "reliability": float 0.0-1.0 representing trustworthiness (Reuters=0.95, Twitter=0.45, arxiv=0.88, Reddit=0.5)
- "freshness": "real_time", "daily", "weekly", or "static"
- "data_type": what information it carries (e.g. "news", "academic_papers", "code", "reviews", "people_data")
- "bias_level": "low", "medium", or "high"
- "best_for": JSON array of 2-3 use cases this source excels at"""

    edge_instructions = 'Include MORE_RELIABLE_THAN edges where clearly applicable (e.g. reuters more reliable than twitter_x_feed). Include PRIMARY_SOURCE_FOR edges to concept IDs where this is authoritative (e.g. concept:job_market).'

    total_nodes, total_edges = [], []
    for batch in [sources_b1, sources_b2]:
        nodes, edges = gemini_batch(api_key, batch, "source", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} source nodes, {len(total_edges)} edges")




def pass_concepts(api_key: str, graph: dict):
    """Pass 5: Core concept nodes that tie everything together."""
    print("\n[Pass 5] Concepts (cross-linking hub nodes)...")

    prompt = f"""You are seeding a knowledge graph for an AI agent. Generate concept nodes that act as semantic hubs connecting industries, platforms, tactics, and sources.

Concepts are abstract ideas that multiple nodes relate to. They allow the graph to be traversed by meaning, not just by name.

For each concept node (node_type: "concept"), include in meta:
- "description": what this concept means in an AI agent context
- "related_domains": list of domain names this concept belongs to

Generate concepts like: lead_gen, outreach, content_marketing, seo, hiring, fundraising, market_research, 
competitive_intelligence, customer_success, product_launch, networking, community_building, 
brand_awareness, data_analysis, automation, bug_finding, code_review, 3d_modeling, video_editing,
cold_email, warm_introduction, inbound, outbound, saas_metrics, growth_hacking, and more.

Connect concept nodes to each other with RELATED_TO edges where meaningful.

{SCHEMA_REMINDER}
"""
    nodes, edges = parse_response(call_gemini(api_key, prompt))
    merge_into_graph(graph, nodes, edges)
    print(f"  + {len(nodes)} concept nodes, {len(edges)} edges")


def pass_game_engines_and_dev_tools(api_key: str, graph: dict):
    print("\n[Pass 6] Game engines + dev tools...")
    tools_b1 = ["unreal_engine", "unity", "godot", "gamemaker_studio", "rpg_maker", "cry_engine", "o3de", "pygame", "love2d", "bevy"]
    tools_b2 = ["visual_studio", "visual_studio_code", "xcode", "android_studio", "intellij_idea", "pycharm", "webstorm", "rider", "clion", "eclipse"]
    tools_b3 = ["cmake", "gradle", "webpack", "vite", "docker", "kubernetes", "terraform", "ansible", "postman", "insomnia"]

    meta = """\
- "platform_type": "desktop_app" or "cli"
- "nature": what this tool is and who uses it (1 sentence)
- "primary_use_cases": JSON array of 3-5 things an AI agent or developer does with it
- "target": app name or CLI command to invoke it"""

    edge_instructions = 'Include BEST_FOR edges to relevant industry IDs (e.g. "industry:gaming", "industry:saas", "industry:ai_ml"). Include RELATED_TO edges between closely related tools.'

    total_nodes, total_edges = [], []
    for batch in [tools_b1, tools_b2, tools_b3]:
        nodes, edges = gemini_batch(api_key, batch, "platform", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} nodes, {len(total_edges)} edges")


def pass_3d_and_creative_tools(api_key: str, graph: dict):
    print("\n[Pass 7] 3D modelling + creative tools...")
    tools_b1 = ["blender", "maya", "3ds_max", "cinema_4d", "zbrush", "substance_painter", "substance_designer", "houdini", "modo", "mudbox"]
    tools_b2 = ["photoshop", "illustrator", "after_effects", "premiere_pro", "davinci_resolve", "final_cut_pro", "lightroom", "indesign", "sketch", "affinity_designer"]
    tools_b3 = ["figma", "framer", "spline", "tinkercad", "openscad", "rhino3d", "sketchup", "archicad", "revit", "lumion"]
    tools_b4 = ["stable_diffusion", "midjourney", "runway_ml", "kling", "sora", "elevenlabs", "suno", "udio", "topaz_video_ai", "frame_io"]

    meta = """\
- "platform_type": "desktop_app" or "web"
- "nature": what this creative tool does and who uses it (1 sentence)
- "primary_use_cases": JSON array of 3-5 specific things an AI agent can do or observe using this tool
- "output_types": JSON array of what this tool produces (e.g. ["3d_model", "texture", "animation", "render"])
- "target": app name or URL"""

    edge_instructions = 'Include BEST_FOR edges to relevant industries: "industry:gaming", "industry:media_entertainment", "industry:architecture", "industry:animation_vfx". Include PART_OF edges to "app_category:3d_modeling" or "app_category:design_tool" as appropriate.'

    total_nodes, total_edges = [], []
    for batch in [tools_b1, tools_b2, tools_b3, tools_b4]:
        nodes, edges = gemini_batch(api_key, batch, "platform", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} nodes, {len(total_edges)} edges")


def pass_electronics_and_engineering(api_key: str, graph: dict):
    print("\n[Pass 8] Electronics + engineering tools...")
    tools_b1 = ["kicad", "eagle_pcb", "altium_designer", "circuit_maker", "ltspice", "multisim", "proteus", "fritzing", "easyeda", "fusion_360"]
    tools_b2 = ["autocad", "solidworks", "catia", "creo", "siemens_nx", "ansys", "abaqus", "matlab", "simulink", "labview"]
    tools_b3 = ["arduino_ide", "platformio", "stm32cubeide", "keil_mdk", "openocd", "quartus", "vivado", "modelsin", "comsol", "openfoam"]

    meta = """\
- "platform_type": "desktop_app" or "web"
- "nature": what this engineering tool does and what domain it covers (1 sentence)
- "primary_use_cases": JSON array of 3-5 things engineers or an AI agent do with it
- "domain": the engineering discipline — "electronics", "mechanical", "simulation", "embedded", "pcb", "cad"
- "target": app name"""

    edge_instructions = 'Include BEST_FOR edges to: "industry:hardware_electronics", "industry:automotive", "industry:aerospace", "industry:manufacturing", "industry:biotech" as relevant. Include RELATED_TO edges between complementary tools.'

    total_nodes, total_edges = [], []
    for batch in [tools_b1, tools_b2, tools_b3]:
        nodes, edges = gemini_batch(api_key, batch, "platform", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} nodes, {len(total_edges)} edges")


def pass_audio_and_video_production(api_key: str, graph: dict):
    print("\n[Pass 9] Audio + video production tools...")
    tools_b1 = ["ableton_live", "logic_pro", "fl_studio", "pro_tools", "cubase", "reaper", "garageband", "audacity", "audition", "bitwig"]
    tools_b2 = ["obs_studio", "streamlabs", "ecamm_live", "vmix", "wirecast", "resolume", "touchdesigner", "notch", "madmapper", "modul8"]
    tools_b3 = ["splice", "native_instruments", "izotope", "waves_plugins", "soundtoys", "plugin_alliance", "arturia", "overbridge", "loopback", "blackhole_audio"]

    meta = """\
- "platform_type": "desktop_app" or "web"
- "nature": what this audio/video tool does and who uses it (1 sentence)
- "primary_use_cases": JSON array of 3-4 specific production tasks
- "media_type": "audio", "video", "live_streaming", "visual_performance", or "audio_plugin"
- "target": app name"""

    edge_instructions = 'Include BEST_FOR edges to: "industry:media_entertainment", "industry:music_production", "industry:gaming", "industry:podcast". Include PART_OF edges to "app_category:media_player" or "app_category:screen_recorder" where applicable.'

    total_nodes, total_edges = [], []
    for batch in [tools_b1, tools_b2, tools_b3]:
        nodes, edges = gemini_batch(api_key, batch, "platform", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} nodes, {len(total_edges)} edges")


def pass_science_and_data_tools(api_key: str, graph: dict):
    print("\n[Pass 10] Science + data tools...")
    tools_b1 = ["jupyter_notebook", "rstudio", "spss", "sas", "stata", "tableau", "power_bi", "looker", "metabase", "grafana"]
    tools_b2 = ["excel", "google_sheets", "airtable", "databricks", "snowflake", "bigquery", "redshift", "dbt", "airflow", "prefect"]
    tools_b3 = ["tensorflow", "pytorch", "huggingface", "weights_and_biases", "mlflow", "ray", "dask", "spark", "kafka", "elasticsearch"]

    meta = """\
- "platform_type": "desktop_app", "web", "cli", or "library"
- "nature": what this data/science tool does and who uses it (1 sentence)
- "primary_use_cases": JSON array of 3-5 things a data scientist or AI agent does with it
- "data_domain": "ml_training", "data_viz", "data_pipeline", "statistics", "database", "monitoring"
- "target": app name, URL, or pip install name"""

    edge_instructions = 'Include BEST_FOR edges to: "industry:ai_ml", "industry:fintech", "industry:healthtech", "industry:saas", "industry:consulting". Include RELATED_TO edges between complementary tools.'

    total_nodes, total_edges = [], []
    for batch in [tools_b1, tools_b2, tools_b3]:
        nodes, edges = gemini_batch(api_key, batch, "platform", meta, edge_instructions)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} nodes, {len(total_edges)} edges")


def pass_missing_industries(api_key: str, graph: dict):
    print("\n[Pass 11] Missing industries...")
    missing_b1 = ["gaming", "animation_vfx", "music_production", "film_production", "architecture", "hardware_electronics", "semiconductors", "robotics", "space_tech", "quantum_computing"]
    missing_b2 = ["fashion", "food_beverage", "sports_fitness", "travel_tourism", "events_entertainment", "education_k12", "higher_education", "social_impact", "climate_tech", "web3_crypto"]
    missing_b3 = ["podcast", "creator_economy", "influencer_marketing", "prop_tech", "legal_tech", "hr_tech", "supply_chain_tech", "maritime", "mining", "energy_utilities"]

    meta = """\
- "description": what this industry is and who the key players are (1-2 sentences)
- "b2b_or_b2c": "b2b", "b2c", or "both"
- "typical_goals": JSON array of 3-4 common business goals in this industry
- "key_job_titles": JSON array of key decision-maker roles
- "relevant_tools": JSON array of 3-5 software tool names commonly used in this industry"""

    total_nodes, total_edges = [], []
    for batch in [missing_b1, missing_b2, missing_b3]:
        nodes, edges = gemini_batch(api_key, batch, "industry", meta)
        merge_into_graph(graph, nodes, edges)
        total_nodes.extend(nodes); total_edges.extend(edges)
        time.sleep(1)
    print(f"  + {len(total_nodes)} nodes, {len(total_edges)} edges")


# ─────────────────────────────────────────────
# MAIN
# ─────────────────────────────────────────────

ALL_PASSES = {
    "app_categories": pass_app_categories,
    "industries": pass_industries,
    "platforms": pass_platforms,
    "tactics": pass_domains_and_tactics,
    "sources": pass_information_sources,
    "concepts": pass_concepts,
    # Expansion passes — add more any time
    "game_engines": pass_game_engines_and_dev_tools,
    "creative_tools": pass_3d_and_creative_tools,
    "electronics": pass_electronics_and_engineering,
    "audio_video": pass_audio_and_video_production,
    "science_data": pass_science_and_data_tools,
    "more_industries": pass_missing_industries,
}


def main():
    parser = argparse.ArgumentParser(description="Bootstrap Momentum Knowledge Graph via Gemini API")
    parser.add_argument("--api-key", required=True, help="Gemini API key from aistudio.google.com")
    parser.add_argument("--passes", nargs="*", default=list(ALL_PASSES.keys()),
                        help=f"Which passes to run. Options: {list(ALL_PASSES.keys())}")
    parser.add_argument("--fresh", action="store_true",
                        help="Start fresh (wipe existing graph)")
    args = parser.parse_args()

    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("  Momentum Knowledge Graph Bootstrap")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print(f"  Graph path : {GRAPH_PATH}")
    print(f"  Passes     : {args.passes}")

    graph = {"version": "3.0.0", "nodes": {}, "edges": []} if args.fresh else load_graph()
    print(f"  Loaded     : {len(graph['nodes'])} nodes, {len(graph['edges'])} edges")

    for pass_name in args.passes:
        if pass_name not in ALL_PASSES:
            print(f"  ✗ Unknown pass: {pass_name}")
            continue
        t0 = time.time()
        ALL_PASSES[pass_name](args.api_key, graph)
        save_graph(graph)
        print(f"  ⏱  {time.time() - t0:.1f}s")
        time.sleep(1)  # be nice to the API

    print("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print(f"  DONE: {len(graph['nodes'])} nodes, {len(graph['edges'])} edges")
    print(f"  Graph saved to: {GRAPH_PATH}")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")


if __name__ == "__main__":
    main()
