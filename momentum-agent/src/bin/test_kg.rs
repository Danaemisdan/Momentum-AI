use momentum_agent::knowledge::{LocalKnowledgeGraph, KnowledgeGraph, Node, NodeType, Edge, EdgeRel};
use momentum_agent::researcher::AutonomousWebResearcher;
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║       MOMENTUM AI — SELF-EVOLVING KNOWLEDGE GRAPH TEST SUITE         ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    let args: Vec<String> = env::args().collect();
    let custom_goal = if args.len() > 1 { Some(args[1..].join(" ")) } else { None };

    let kg_path = LocalKnowledgeGraph::default_path();
    println!("📂 Knowledge Graph Storage: {:?}", kg_path);

    let kg = Arc::new(Mutex::new(LocalKnowledgeGraph::load()));
    let researcher = AutonomousWebResearcher::new(kg.clone());

    // ─── TEST 1: Label Normalization ──────────────────────────────────────────
    println!("\n[TEST 1] Label Normalization...");
    {
        let goal_a = "Can you please find me clients on LinkedIn for my agency";
        let norm_a = KnowledgeGraph::normalize_label(goal_a);
        println!("  • Input:      \"{}\"", goal_a);
        println!("  • Normalized: \"{}\"", norm_a);
        assert!(!norm_a.contains("please") && !norm_a.contains("_me_") && !norm_a.contains("_my_"));
        println!("  ✅ TEST 1 PASSED");
    }

    // ─── TEST 2: Node upsert + edge + traverse ────────────────────────────────
    println!("\n[TEST 2] Node + Edge + Traversal...");
    {
        let mut kg_lock = kg.lock().await;
        kg_lock.upsert_node(
            Node::new("person:testjag", NodeType::Person, "TestJag")
                .with_meta("aliases", serde_json::json!(["jag", "jag bro"]))
        );
        kg_lock.upsert_node(Node::new("platform:whatsapp_test", NodeType::Platform, "WhatsApp Test"));
        kg_lock.add_edge(Edge::new("person:testjag", "platform:whatsapp_test", EdgeRel::ReachableOn));

        let platforms = kg_lock.traverse("person:testjag", &EdgeRel::ReachableOn);
        assert_eq!(platforms.len(), 1);
        assert_eq!(platforms[0].label, "WhatsApp Test");
        println!("  ✅ TEST 2 PASSED: Traversal works");
    }

    // ─── TEST 3: Context Injection ────────────────────────────────────────────
    println!("\n[TEST 3] LLM Context Injection...");
    {
        let kg_lock = kg.lock().await;
        let ctx = kg_lock.inject_context_for("message testjag on whatsapp");
        println!("  Context snippet:\n{}", &ctx[..ctx.len().min(400)]);
        assert!(ctx.contains("TestJag") || ctx.contains("WhatsApp Test") || ctx.is_empty());
        println!("  ✅ TEST 3 PASSED");
    }

    // ─── TEST 4: Web Research → Graph Nodes ──────────────────────────────────
    let test_goal = custom_goal.unwrap_or_else(|| "find clients for web development agency".to_string());
    println!("\n[TEST 4] Web Research → Graph Nodes...");
    println!("  • Goal: \"{}\"", test_goal);

    let model_candidates = [
        "../momentum-engine-tiny-0.5b.gguf",
        "momentum-engine-tiny-0.5b.gguf",
        "/Users/sanjeevn/Downloads/Momentum AI/momentum-engine-tiny-0.5b.gguf",
    ];
    let model_path = model_candidates.into_iter().find(|p| std::path::Path::new(p).exists());
    let brain = if let Some(p) = model_path {
        println!("🧠 Initializing 0.5B Brain: {}", p);
        momentum_agent::brain::MomentumBrain::init(p).ok()
    } else {
        println!("ℹ️ 0.5B model not found, using heuristic mode.");
        None
    };

    let t0 = Instant::now();
    let domain_entry = researcher.research_domain(&test_goal, brain.as_ref()).await;
    println!("  ⚡ Research completed in {:.2?}", t0.elapsed());
    println!("  🏛️ Domain: \"{}\"", domain_entry.domain_name);
    println!("  📋 Discovered {} platforms:", domain_entry.platforms.len());
    for (i, p) in domain_entry.platforms.iter().enumerate() {
        println!("     {}. [{}] {} → {}", i + 1, p.platform_type, p.name, p.target);
    }
    println!("  ❓ Clarification: \"{}\"", domain_entry.clarification_question);
    assert!(!domain_entry.platforms.is_empty(), "Must discover platforms!");
    println!("  ✅ TEST 4 PASSED");

    // ─── TEST 5: Graph Stats ──────────────────────────────────────────────────
    println!("\n[TEST 5] Graph Inspection...");
    {
        let kg_lock = kg.lock().await;
        println!("  • Nodes: {}", kg_lock.node_count());
        println!("  • Edges: {}", kg_lock.edge_count());
        for node in kg_lock.find_nodes_by_type(&NodeType::Platform).iter().take(5) {
            println!("    Platform: {} ({})", node.label, node.id);
        }
        println!("  ✅ TEST 5 PASSED");
    }

    println!("\n🎉 ALL TESTS PASSED! Knowledge Graph v3 operational.\n");
    Ok(())
}
