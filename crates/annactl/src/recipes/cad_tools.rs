// Beta.175: CAD and Engineering Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct CADToolsRecipe;

#[derive(Debug, PartialEq)]
enum CADToolsOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl CADToolsOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            CADToolsOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            CADToolsOperation::ListTools
        } else {
            CADToolsOperation::Install
        }
    }
}

impl CADToolsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("freecad") || input_lower.contains("librecad")
            || input_lower.contains("kicad") || input_lower.contains("openscad")
            || (input_lower.contains("pcb") && input_lower.contains("design"))
            || (input_lower.contains("electronics") && input_lower.contains("design"))
            || input_lower.contains("cad") && (
                input_lower.contains("install") || input_lower.contains("setup")
                || input_lower.contains("3d") || input_lower.contains("2d")
                || input_lower.contains("tool") || input_lower.contains("editor")
            );
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = CADToolsOperation::detect(user_input);
        match operation {
            CADToolsOperation::Install => Self::build_install_plan(telemetry),
            CADToolsOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            CADToolsOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("freecad") { "freecad" }
        else if input_lower.contains("librecad") { "librecad" }
        else if input_lower.contains("kicad") { "kicad" }
        else if input_lower.contains("openscad") { "openscad" }
        else if input_lower.contains("2d") { "librecad" }
        else if input_lower.contains("pcb") || input_lower.contains("electronics") { "kicad" }
        else if input_lower.contains("3d") { "freecad" }
        else { "freecad" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "freecad" => ("FreeCAD", "freecad", "Open-source parametric 3D CAD modeler for mechanical engineering and product design"),
            "librecad" => ("LibreCAD", "librecad", "Open-source 2D CAD application for creating technical drawings"),
            "kicad" => ("KiCad", "kicad", "Open-source electronics design automation suite for schematic capture and PCB layout"),
            "openscad" => ("OpenSCAD", "openscad", "Script-based 3D CAD modeler for creating solid 3D models programmatically"),
            _ => ("FreeCAD", "freecad", "3D parametric CAD modeler", ),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("cad_tools.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);

        let notes = match tool {
            "freecad" => format!("{} installed. {}. Launch from app menu or run 'freecad' in terminal. Supports STEP, IGES, STL, and other 3D formats.", tool_name, description),
            "librecad" => format!("{} installed. {}. Launch from app menu or run 'librecad' in terminal. Supports DXF and DWG formats.", tool_name, description),
            "kicad" => format!("{} installed. {}. Launch from app menu or run 'kicad' in terminal. Includes schematic editor (Eeschema) and PCB editor (Pcbnew).", tool_name, description),
            "openscad" => format!("{} installed. {}. Launch from app menu or run 'openscad' in terminal. Edit .scad script files to create 3D models.", tool_name, description),
            _ => format!("{} installed. {}", tool_name, description),
        };

        Ok(ActionPlan {
            analysis: format!("Installing {} CAD tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("cad_tools_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("cad_tools.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking installed CAD tools".to_string(),
            goals: vec!["List installed CAD software".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-cad-tools".to_string(),
                    description: "List CAD tools".to_string(),
                    command: "pacman -Q freecad librecad kicad openscad qcad 2>/dev/null || echo 'No CAD tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed CAD and engineering design software".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("cad_tools_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("cad_tools.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available CAD and engineering tools".to_string(),
            goals: vec!["List available CAD software for Arch Linux".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available CAD tools".to_string(),
                    command: r#"echo 'CAD and Engineering Tools for Arch Linux:

3D CAD (Parametric):
- FreeCAD (official) - Open-source parametric 3D CAD modeler with modular architecture
- SolveSpace (official) - Parametric 2D/3D CAD with constraint-based modeling
- BRL-CAD (AUR) - Constructive solid geometry (CSG) modeling system
- OpenCascade (official) - CAD kernel used by FreeCAD and other CAD software

2D CAD:
- LibreCAD (official) - 2D CAD based on QCAD Community Edition
- QCAD (official) - Professional 2D CAD system with DXF/DWG support
- Draftsight (AUR) - Professional-grade 2D CAD (DWG/DXF native)

Electronics CAD (EDA):
- KiCad (official) - Complete electronics design automation suite
- gEDA (official) - Electronic design automation toolset
- Fritzing (AUR) - Electronics prototyping tool for breadboards, schematics, PCBs
- ngspice (official) - Mixed-level/mixed-signal circuit simulator

Script-Based 3D CAD:
- OpenSCAD (official) - Programmable solid 3D CAD modeler using scripting language
- ImplicitCAD (AUR) - Programmatic CAD system with math-based modeling
- CadQuery (AUR) - Python-based parametric CAD library

Mechanical Engineering:
- FreeCAD (official) - Full-featured with FEM analysis, robot simulation, CAM
- Gmsh (official) - 3D finite element mesh generator with built-in CAD engine
- CalculiX (AUR) - Finite element analysis (FEA) solver
- ElmerFEM (AUR) - Multiphysics simulation software

Architecture:
- Blender (official) - General 3D modeling with architecture add-ons (ArchiPack)
- Sweet Home 3D (AUR) - Interior design application for home floor plans
- LibreCAD (official) - Architectural 2D drawings and floor plans

PCB Design:
- KiCad (official) - Industry-standard open-source PCB design
- Eagle (AUR) - Professional PCB design (Autodesk, freemium)
- Horizon EDA (AUR) - Modern PCB design tool with git integration
- gEDA PCB (official) - Interactive printed circuit board editor

Circuit Simulation:
- ngspice (official) - SPICE simulator for analog/mixed-signal circuits
- Qucs (AUR) - Integrated circuit simulator with GUI
- LTspice (via Wine) - Professional SPICE simulator from Analog Devices
- Xyce (AUR) - SPICE-compatible parallel circuit simulator

3D Modeling (General):
- Blender (official) - Professional 3D creation suite, can export CAD formats
- Wings3D (AUR) - Subdivision modeler for organic shapes
- MeshLab (official) - Processing and editing 3D triangular meshes

File Format Conversion:
- meshconv (AUR) - Convert between 3D mesh formats (STL, OBJ, PLY)
- OpenCascade (official) - CAD kernel with STEP, IGES support
- FreeCAD (official) - Import/export STEP, IGES, STL, OBJ, DXF, SVG

Comparison by Use Case:

Mechanical Engineering / Product Design:
- FreeCAD: Best free option, parametric modeling, assemblies, FEM analysis
- SolveSpace: Best for constraint-based parametric modeling
- OpenSCAD: Best for programmable/parametric models from scripts

2D Technical Drawings:
- LibreCAD: Best free 2D CAD, DXF native format
- QCAD: Best for professional 2D drafting with plugins
- Draftsight: Best DWG compatibility (proprietary)

Electronics / PCB Design:
- KiCad: Best open-source PCB design, professional grade
- Eagle: Best for established designs and community (freemium)
- Fritzing: Best for beginners and prototyping

Programmable CAD:
- OpenSCAD: Best for script-based modeling with CSG operations
- CadQuery: Best for Python-based parametric CAD
- ImplicitCAD: Best for mathematical/implicit modeling

Key Features:

FreeCAD:
- Parametric modeling with constraint solver
- Assembly workbench for complex products
- FEM analysis (CalculiX integration)
- Path workbench for CAM/CNC
- Robot simulation, ship design, architecture
- Python scripting and macro support
- Supports STEP, IGES, STL, OBJ, DXF, SVG

LibreCAD:
- 2D only, native DXF format
- Layer management, blocks, hatching
- Dimensioning and text annotations
- Snap grid and construction lines
- Plugin architecture

KiCad:
- Schematic capture (Eeschema)
- PCB layout (Pcbnew) with 3D viewer
- Symbol and footprint libraries
- Design rule checking (DRC)
- Gerber and drill file generation
- SPICE simulation integration
- 3D model import (STEP/WRL)

OpenSCAD:
- Script-based CSG modeling
- 2D to 3D extrusion and rotation
- Boolean operations (union, difference, intersection)
- Parametric designs via variables
- Import STL, DXF; export STL, OFF, AMF, 3MF
- Animation support

File Format Support:

STEP (.step, .stp):
- FreeCAD: Full read/write support
- OpenCascade: Native format
- Most professional CAD tools support

IGES (.iges, .igs):
- FreeCAD: Read/write support
- Older standard, less common now

STL (.stl):
- All tools: Universal 3D printing format
- FreeCAD, OpenSCAD, Blender: Full support

DXF (.dxf):
- LibreCAD: Native format
- FreeCAD: Read/write
- Universal 2D exchange format

DWG (.dwg):
- QCAD: Native support
- Draftsight: Native support
- LibreCAD: Via converter

Learning Resources:

FreeCAD:
- Official wiki with comprehensive tutorials
- YouTube channels: MangoJelly Solutions, Joko Engineeringhelp
- FreeCAD forum and community

LibreCAD:
- Official documentation and user manual
- YouTube tutorials for 2D drafting

KiCad:
- Official documentation and getting started guide
- Contextual Engineering YouTube channel
- KiCad forum with active community

OpenSCAD:
- Cheat sheet and language reference
- OpenSCAD tutorial on official website
- Thingiverse has many OpenSCAD examples

Workflow Integration:

Design to Manufacturing:
1. Create 3D model in FreeCAD
2. Export to STEP for collaboration
3. Export to STL for 3D printing
4. Use Path workbench for CNC machining

PCB Production:
1. Design schematic in KiCad Eeschema
2. Assign footprints and create netlist
3. Layout PCB in Pcbnew
4. Generate Gerber files for fab house
5. Order from OSH Park, JLCPCB, PCBWay

Simulation Workflow:
1. Create geometry in FreeCAD or OpenSCAD
2. Export mesh for FEM
3. Run analysis in CalculiX or ElmerFEM
4. Visualize results in ParaView

Performance Considerations:

FreeCAD:
- Complex assemblies can be slow
- Use simplified representations for large assemblies
- Enable level of detail (LOD) for better performance

LibreCAD:
- Lightweight, fast for 2D work
- Handles large drawings well

KiCad:
- Modern architecture, good performance
- 3D viewer can be GPU intensive
- Large boards benefit from 64-bit build

OpenSCAD:
- Render time depends on complexity
- Preview mode for fast iteration
- Final render for accurate geometry

Tips and Best Practices:

FreeCAD:
- Use Sketcher for 2D constraints before 3D modeling
- Name features descriptively in model tree
- Use Part Design for single-part models
- Use Assembly4 or A2plus workbench for assemblies
- Save frequently, use version control for projects

LibreCAD:
- Set up layers before starting
- Use blocks for repeated elements
- Snap to grid for precision
- Learn keyboard shortcuts for efficiency

KiCad:
- Start with schematic, annotate before PCB
- Use design rules from manufacturer early
- Run DRC frequently during layout
- Keep ground plane solid, minimize splits
- Use filled zones for power and ground

OpenSCAD:
- Write modular, reusable functions
- Use variables for parameters
- Comment complex operations
- Use $fn, $fa, $fs for circle resolution
- Preview before final render

Common Issues:

FreeCAD:
- Topological naming problem: Use Realthunder branch or 0.21+
- Missing features: Check if correct workbench is active
- Crashes: Save often, keep backup files

LibreCAD:
- Font rendering: Install additional fonts
- DWG support: Use converter plugins
- Printing: Adjust print scale and paper size

KiCad:
- Library management: Use global and project-specific libraries
- 3D models: Download from KiCad official packages
- Gerber generation: Verify with gerbv before manufacturing

OpenSCAD:
- Slow rendering: Reduce $fn value, simplify geometry
- Manifold errors: Check for overlapping faces
- Export issues: Ensure model is valid before export

System Requirements:

FreeCAD:
- Moderate CPU, benefits from multiple cores
- 8GB+ RAM recommended for assemblies
- GPU with OpenGL 3.3+ for 3D view

LibreCAD:
- Lightweight, runs on most systems
- 2GB RAM sufficient for typical use

KiCad:
- Moderate system requirements
- GPU recommended for 3D viewer
- 4GB+ RAM for complex boards

OpenSCAD:
- CPU-intensive for rendering
- Single-threaded, benefits from high clock speed
- RAM usage depends on model complexity

Community and Support:

FreeCAD:
- Active forum: forum.freecadweb.org
- IRC: #freecad on Libera.Chat
- Reddit: r/FreeCAD

LibreCAD:
- Forum: librecad.org/cms/home.html
- IRC: #librecad on Libera.Chat

KiCad:
- Forum: forum.kicad.info
- Discord and IRC available
- Reddit: r/KiCad

OpenSCAD:
- Mailing list and forum
- IRC: #openscad on Libera.Chat
- GitHub for bug reports'"#.to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "CAD and engineering design tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("cad_tools_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(CADToolsRecipe::matches_request("install freecad"));
        assert!(CADToolsRecipe::matches_request("install librecad"));
        assert!(CADToolsRecipe::matches_request("setup kicad"));
        assert!(CADToolsRecipe::matches_request("install openscad"));
        assert!(CADToolsRecipe::matches_request("install 3d cad"));
        assert!(CADToolsRecipe::matches_request("install pcb design"));
        assert!(!CADToolsRecipe::matches_request("what is freecad"));
    }

    #[test]
    fn test_install_plan_freecad() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install freecad".to_string());
        let plan = CADToolsRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
        assert!(plan.notes_for_user.contains("FreeCAD"));
    }

    #[test]
    fn test_install_plan_kicad() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install kicad".to_string());
        let plan = CADToolsRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
        assert!(plan.notes_for_user.contains("KiCad"));
    }

    #[test]
    fn test_detect_tool() {
        assert_eq!(CADToolsRecipe::detect_tool("install freecad"), "freecad");
        assert_eq!(CADToolsRecipe::detect_tool("setup kicad"), "kicad");
        assert_eq!(CADToolsRecipe::detect_tool("install 2d cad"), "librecad");
        assert_eq!(CADToolsRecipe::detect_tool("install pcb design"), "kicad");
    }
}
