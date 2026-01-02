//! Settings-related module declarations
//! All settings modules (v0.0.554 onwards)

// Submodule declarations
mod core;
mod network_sync;
mod api_interface;
mod system;
mod runtime;
mod architecture;
mod data_processing;
mod version_control;
mod collection;
mod documentation;
mod governance;
mod geography;
mod community;

// Re-export all modules to maintain the public API

// Settings core modules (v0.0.554-563)
pub use core::unified_settings;
pub use core::settings_persistence;
pub use core::settings_migration;
pub use core::settings_validation;
pub use core::settings_export;
pub use core::settings_cli;
pub use core::settings_watcher;
pub use core::settings_diff;
pub use core::settings_presets;
pub use core::settings_history;

// Settings network/sync modules (v0.0.564-577)
pub use network_sync::settings_sync;
pub use network_sync::settings_profiles;
pub use network_sync::settings_search;
pub use network_sync::settings_notifications;
pub use network_sync::settings_scheduler;
pub use network_sync::settings_templates;
pub use network_sync::settings_constraints;
pub use network_sync::settings_hooks;
pub use network_sync::settings_wizard;
pub use network_sync::settings_audit;
pub use network_sync::settings_orchestrator;
pub use network_sync::settings_backup;
pub use network_sync::settings_restore;
pub use network_sync::settings_analytics;

// Settings API/interface modules (v0.0.578-592)
pub use api_interface::settings_recommendations;
pub use api_interface::settings_dashboard;
pub use api_interface::settings_api;
pub use api_interface::settings_events;
pub use api_interface::settings_permissions;
pub use api_interface::settings_diagnostics;
pub use api_interface::settings_metrics;
pub use api_interface::settings_logging;
pub use api_interface::settings_cache;
pub use api_interface::settings_transactions;
pub use api_interface::settings_versioning;
pub use api_interface::settings_throttling;
pub use api_interface::settings_middleware;
pub use api_interface::settings_observer;
pub use api_interface::settings_snapshot;

// Settings system modules (v0.0.593-610)
pub use system::settings_lock;
pub use system::settings_encryption;
pub use system::settings_inheritance;
pub use system::settings_query;
pub use system::settings_validator_chain;
pub use system::settings_transformer;
pub use system::settings_resolver;
pub use system::settings_aggregator;
pub use system::settings_comparator;
pub use system::settings_serializer;
pub use system::settings_router;
pub use system::settings_compiler;
pub use system::settings_linker;
pub use system::settings_bundler;
pub use system::settings_deployer;
pub use system::settings_monitor;
pub use system::settings_reporter;
pub use system::settings_task_scheduler;

// Settings runtime modules (v0.0.611-627)
pub use runtime::settings_queue;
pub use runtime::settings_worker;
pub use runtime::settings_executor;
pub use runtime::settings_pipeline;
pub use runtime::settings_processor;
pub use runtime::settings_handler;
pub use runtime::settings_dispatcher;
pub use runtime::settings_coordinator;
pub use runtime::settings_controller;
pub use runtime::settings_service;
pub use runtime::settings_manager;
pub use runtime::settings_registry;
pub use runtime::settings_index;
pub use runtime::settings_catalog;
pub use runtime::settings_gateway;
pub use runtime::settings_proxy;
pub use runtime::settings_facade;

// Settings architecture modules (v0.0.628-642)
pub use architecture::settings_adapter;
pub use architecture::settings_bridge;
pub use architecture::settings_connector;
pub use architecture::settings_provider;
pub use architecture::settings_consumer;
pub use architecture::settings_subscriber;
pub use architecture::settings_publisher;
pub use architecture::settings_broadcaster;
pub use architecture::settings_listener;
pub use architecture::settings_poller;
pub use architecture::settings_tracker;
pub use architecture::settings_notifier;
pub use architecture::settings_report_generator;
pub use architecture::settings_inspector;
pub use architecture::settings_analyzer;

// Settings data processing modules (v0.0.643-658)
pub use data_processing::settings_sanitizer;
pub use data_processing::settings_formatter;
pub use data_processing::settings_normalizer;
pub use data_processing::settings_parser;
pub use data_processing::settings_renderer;
pub use data_processing::settings_encoder;
pub use data_processing::settings_decoder;
pub use data_processing::settings_converter;
pub use data_processing::settings_mapper;
pub use data_processing::settings_binder;
pub use data_processing::settings_extractor;
pub use data_processing::settings_injector;
pub use data_processing::settings_merger;
pub use data_processing::settings_splitter;
pub use data_processing::settings_cloner;
pub use data_processing::settings_archiver;

// Settings version control modules (v0.0.659-676)
pub use version_control::settings_restorer;
pub use version_control::settings_versioner;
pub use version_control::settings_differ;
pub use version_control::settings_patcher;
pub use version_control::settings_graph;
pub use version_control::settings_resolution;
pub use version_control::settings_validator_hub;
pub use version_control::settings_transform;
pub use version_control::settings_normalization;
pub use version_control::settings_denormalization;
pub use version_control::settings_indexer;
pub use version_control::settings_query_engine;
pub use version_control::settings_aggregation;
pub use version_control::settings_projector;
pub use version_control::settings_selector;
pub use version_control::settings_filter;
pub use version_control::settings_sorter;
pub use version_control::settings_grouper;

// Settings collection modules (v0.0.677-692)
pub use collection::settings_reducer;
pub use collection::settings_partitioner;
pub use collection::settings_flattener;
pub use collection::settings_expander;
pub use collection::settings_iterator;
pub use collection::settings_collector;
pub use collection::settings_zipper;
pub use collection::settings_scanner;
pub use collection::settings_finder;
pub use collection::settings_counter;
pub use collection::settings_matcher;
pub use collection::settings_validator;
pub use collection::settings_comparer;
pub use collection::settings_combiner;
pub use collection::settings_auditor;
pub use collection::settings_chronicle;

// Settings documentation modules (v0.0.693-720)
pub use documentation::settings_ledger;
pub use documentation::settings_diary;
pub use documentation::settings_folio;
pub use documentation::settings_album;
pub use documentation::settings_dossier;
pub use documentation::settings_portfolio;
pub use documentation::settings_catalog_v2;
pub use documentation::settings_compendium;
pub use documentation::settings_anthology;
pub use documentation::settings_archive_v2;
pub use documentation::settings_repertoire;
pub use documentation::settings_gazette;
pub use documentation::settings_almanac;
pub use documentation::settings_bulletin;
pub use documentation::settings_journal;
pub use documentation::settings_memo;
pub use documentation::settings_digest;
pub use documentation::settings_brief;
pub use documentation::settings_summary;
pub use documentation::settings_report;
pub use documentation::settings_notice;
pub use documentation::settings_dispatch;
pub use documentation::settings_communique;
pub use documentation::settings_missive;
pub use documentation::settings_circular;
pub use documentation::settings_directive;
pub use documentation::settings_edict;

// Settings governance modules (v0.0.720-750)
pub use governance::settings_decree;
pub use governance::settings_mandate;
pub use governance::settings_ordinance;
pub use governance::settings_statute;
pub use governance::settings_charter;
pub use governance::settings_constitution;
pub use governance::settings_covenant;
pub use governance::settings_treaty;
pub use governance::settings_protocol;
pub use governance::settings_compact;
pub use governance::settings_accord;
pub use governance::settings_pact;
pub use governance::settings_concordat;
pub use governance::settings_convention;
pub use governance::settings_entente;
pub use governance::settings_alliance;
pub use governance::settings_coalition;
pub use governance::settings_federation;
pub use governance::settings_confederation;
pub use governance::settings_union;
pub use governance::settings_bloc;
pub use governance::settings_sphere;
pub use governance::settings_zone;
pub use governance::settings_domain;
pub use governance::settings_realm;
pub use governance::settings_territory;
pub use governance::settings_province;
pub use governance::settings_region;
pub use governance::settings_district;
pub use governance::settings_county;

// Settings geography modules (v0.0.750-786)
pub use geography::settings_municipality;
pub use geography::settings_borough;
pub use geography::settings_ward;
pub use geography::settings_precinct;
pub use geography::settings_neighborhood;
pub use geography::settings_block;
pub use geography::settings_lot;
pub use geography::settings_parcel;
pub use geography::settings_plot;
pub use geography::settings_tract;
pub use geography::settings_acre;
pub use geography::settings_hectare;
pub use geography::settings_field;
pub use geography::settings_meadow;
pub use geography::settings_pasture;
pub use geography::settings_grove;
pub use geography::settings_orchard;
pub use geography::settings_vineyard;
pub use geography::settings_garden;
pub use geography::settings_nursery;
pub use geography::settings_greenhouse;
pub use geography::settings_conservatory;
pub use geography::settings_arboretum;
pub use geography::settings_botanical;
pub use geography::settings_herbarium;
pub use geography::settings_aquarium;
pub use geography::settings_vivarium;
pub use geography::settings_terrarium;
pub use geography::settings_aviary;
pub use geography::settings_apiary;
pub use geography::settings_butterfly;
pub use geography::settings_sanctuary;
pub use geography::settings_reserve;
pub use geography::settings_refuge;
pub use geography::settings_haven;
pub use geography::settings_retreat;
pub use geography::settings_hideaway;

// Settings community modules (v0.0.787+)
pub use community::settings_enclave;
