(() => {
  const X32_COLORS = [
    "#555c6c", "#dc3e3e", "#3ec462", "#e6c438",
    "#4486f0", "#d24cd6", "#34ceda", "#e2e8f0"
  ];

  const IN_OPTIONS = [
    [0, "AN1-8 (Local)"], [1, "AN9-16 (Local)"], [2, "AN17-24 (Local)"], [3, "AN25-32 (Local)"],
    [4, "A1-8 (AES50-A)"], [5, "A9-16 (AES50-A)"], [6, "A17-24 (AES50-A)"], [7, "A25-32 (AES50-A)"],
    [8, "A33-40 (AES50-A)"], [9, "A41-48 (AES50-A)"],
    [10, "B1-8 (AES50-B)"], [11, "B9-16 (AES50-B)"], [12, "B17-24 (AES50-B)"], [13, "B25-32 (AES50-B)"],
    [14, "B33-40 (AES50-B)"], [15, "B41-48 (AES50-B)"],
    [16, "CARD1-8"], [17, "CARD9-16"], [18, "CARD17-24"], [19, "CARD25-32"]
  ];

  const CARD_OPTIONS = [
    ...IN_OPTIONS,
    [20, "OUT1-8"], [21, "OUT9-16"], [22, "P16 1-8"], [23, "P16 9-16"],
    [24, "AUX1-6/Mon"], [25, "AuxIN1-6/TB"],
    [26, "UOUT1-8"], [27, "UOUT9-16"], [28, "UOUT17-24"], [29, "UOUT25-32"],
    [30, "UOUT33-40"], [31, "UOUT41-48"]
  ];

  const AUX_OPTIONS = [
    [0, "AUX1-4"], [1, "AN1-2"], [2, "AN1-4"], [3, "AN1-6"],
    [4, "A1-2"], [5, "A1-4"], [6, "A1-6"], [7, "B1-2"], [8, "B1-4"], [9, "B1-6"],
    [10, "CARD1-2"], [11, "CARD1-4"], [12, "CARD1-6"]
  ];

  const OUT_OPTIONS = [
    [0, "AN1-4 / AN5-8"], [1, "AN9-12 / AN13-16"], [2, "A1-4 / A5-8"], [3, "B1-4 / B5-8"],
    [4, "CARD1-4 / CARD5-8"], [5, "OUT1-4 / OUT5-8"], [6, "OUT9-12 / OUT13-16"]
  ];

  let mixerState = null;
  let activeBank = "ch1";
  let toolsCatalog = [];
  let jobsList = [];
  let selectedToolId = null;
  let activeCategory = "All";
  let panicArmed = false;
  let panicTimer = null;
  let vscChannels = [];
  let uploadedTemplateContent = null;
  let lastGeneratedRpp = "";
  let lastGeneratedIni = "";
  let drawerLines = [];

  const $ = (id) => document.getElementById(id);

  function colorHex(idx) {
    return X32_COLORS[((idx % 8) + 8) % 8];
  }

  function faderToDbLabel(f) {
    if (f <= 0.001) return "-inf dB";
    let db;
    if (f >= 0.5) db = 40 * f - 30;
    else if (f >= 0.25) db = 80 * f - 50;
    else if (f >= 0.0625) db = 160 * f - 70;
    else db = 480 * f - 90;
    return `${db >= 0 ? "+" : ""}${db.toFixed(1)} dB`;
  }

  async function api(path, method = "GET", body = null) {
    const opts = { method, headers: {} };
    if (body !== null) {
      opts.headers["Content-Type"] = "application/json";
      opts.body = JSON.stringify(body);
    }
    const res = await fetch(path, opts);
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || `HTTP ${res.status}`);
    return data;
  }

  function appendDrawerLog(text) {
    drawerLines.push(text);
    if (drawerLines.length > 150) drawerLines.shift();
    $("drawerSummary").textContent = text;
    $("drawerLogs").textContent = drawerLines.join("\n");
  }

  // --- Tabs ---
  document.querySelectorAll(".tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".tab").forEach((t) => t.classList.remove("active"));
      document.querySelectorAll(".tab-panel").forEach((p) => p.classList.remove("active"));
      btn.classList.add("active");
      $(`tab-${btn.dataset.tab}`).classList.add("active");
    });
  });

  // --- Top bar actions ---
  $("connectBtn").addEventListener("click", async () => {
    try {
      const ip = $("ipInput").value.trim() || "127.0.0.1:10023";
      const transport = $("transportSelect").value;
      const snap = await api("/api/connect", "POST", { ip, transport });
      updateMixerState(snap);
      appendDrawerLog(`Connected to ${snap.ip} (${snap.model})`);
    } catch (e) {
      appendDrawerLog(`Connect failed: ${e.message}`);
    }
  });

  $("discoverBtn").addEventListener("click", async () => {
    try {
      appendDrawerLog("Scanning LAN for X32/Wing/X-Air consoles...");
      const list = await api("/api/discover");
      const banner = $("discoverBanner");
      banner.classList.remove("hidden");
      if (!list.length) {
        banner.innerHTML = `<span>No mixers responded to LAN broadcast.</span><button class="btn btn-sm" onclick="this.parentElement.classList.add('hidden')">Dismiss</button>`;
        return;
      }
      banner.innerHTML = `<strong>Discovered:</strong> ` + list.map((m) =>
        `<button class="btn btn-sm btn-primary" data-pick-ip="${m.addr}">${m.name} (${m.model}) @ ${m.addr}</button>`
      ).join(" ") + `<button class="btn btn-sm" id="closeDiscoverBtn">Close</button>`;
      banner.querySelectorAll("[data-pick-ip]").forEach((b) => {
        b.addEventListener("click", () => {
          $("ipInput").value = b.dataset.pickIp;
          banner.classList.add("hidden");
          $("connectBtn").click();
        });
      });
      $("closeDiscoverBtn").addEventListener("click", () => banner.classList.add("hidden"));
    } catch (e) {
      appendDrawerLog(`Discovery error: ${e.message}`);
    }
  });

  $("startEmuBtn").addEventListener("click", async () => {
    try {
      const job = await api("/api/tools/run", "POST", {
        tool_id: "x32-emulator",
        params: { ip: "127.0.0.1", port: "10023", model: "X32" }
      });
      appendDrawerLog(`Emulator started (${job.job_id})`);
      $("ipInput").value = "127.0.0.1:10023";
      setTimeout(() => $("connectBtn").click(), 450);
    } catch (e) {
      appendDrawerLog(`Start emulator failed: ${e.message}`);
    }
  });

  $("refreshBtn").addEventListener("click", async () => {
    try {
      const snap = await api("/api/refresh", "POST", {});
      updateMixerState(snap);
      appendDrawerLog("Refreshed full console state.");
    } catch (e) {
      appendDrawerLog(`Refresh failed: ${e.message}`);
    }
  });

  $("globalModePill").addEventListener("click", async () => {
    const isPlay = mixerState && mixerState.routing && mixerState.routing.routswitch === 1;
    await toggleVscMode(!isPlay);
  });

  $("quickBackupBtn").addEventListener("click", async () => {
    try {
      const res = await api("/api/mixer/snapshot-save", "POST", { path: "x32_pre_vsc_backup.json" });
      appendDrawerLog(`Saved snapshot (${res.entries} params) -> ${res.path}`);
    } catch (e) {
      appendDrawerLog(`Backup failed: ${e.message}`);
    }
  });

  $("quickRestoreBtn").addEventListener("click", async () => {
    try {
      const res = await api("/api/mixer/snapshot-restore", "POST", { path: "x32_pre_vsc_backup.json" });
      appendDrawerLog(`Restored ${res.restored} params from ${res.path}`);
    } catch (e) {
      appendDrawerLog(`Restore failed: ${e.message}`);
    }
  });

  $("panicBtn").addEventListener("click", async () => {
    const btn = $("panicBtn");
    if (!panicArmed) {
      panicArmed = true;
      btn.textContent = "CONFIRM PANIC!";
      panicTimer = setTimeout(() => {
        panicArmed = false;
        btn.textContent = "PANIC MUTE";
      }, 3000);
      return;
    }
    clearTimeout(panicTimer);
    panicArmed = false;
    btn.textContent = "PANIC MUTE";
    try {
      await api("/api/mixer/panic-mute", "POST", {});
      appendDrawerLog("Smooth panic mute executed.");
    } catch (e) {
      appendDrawerLog(`Panic mute failed: ${e.message}`);
    }
  });

  $("toggleDrawerBtn").addEventListener("click", () => {
    $("drawerLogs").classList.toggle("hidden");
  });

  // --- Mixer Rendering ---
  $("bankPills").querySelectorAll(".bank-btn").forEach((b) => {
    b.addEventListener("click", () => {
      $("bankPills").querySelectorAll(".bank-btn").forEach((x) => x.classList.remove("active"));
      b.classList.add("active");
      activeBank = b.dataset.bank;
      renderMixer();
    });
  });

  function renderMixer() {
    if (!mixerState) return;
    const grid = $("stripsGrid");
    let items = [];
    let kind = "ch";
    if (activeBank === "ch1") items = mixerState.channels.slice(0, 8);
    else if (activeBank === "ch9") items = mixerState.channels.slice(8, 16);
    else if (activeBank === "ch17") items = mixerState.channels.slice(16, 24);
    else if (activeBank === "ch25") items = mixerState.channels.slice(24, 32);
    else if (activeBank === "aux") { items = mixerState.auxins; kind = "auxin"; }
    else if (activeBank === "bus1") { items = mixerState.buses.slice(0, 8); kind = "bus"; }
    else if (activeBank === "bus9") { items = mixerState.buses.slice(8, 16); kind = "bus"; }
    else if (activeBank === "mtx") { items = mixerState.matrices || []; kind = "mtx"; }
    else if (activeBank === "dca") { items = mixerState.dcas; kind = "dca"; }

    grid.innerHTML = items.map((s) => {
      const hex = colorHex(s.color);
      const meterPct = Math.round((s.meter || 0) * 100);
      const linkHtml = s.linked_stereo ? `<span class="link-badge">LINK</span>` : "";
      const panHtml = kind === "ch"
        ? `<input type="range" class="pan-slider" min="0" max="1" step="0.01" value="${s.pan ?? 0.5}" data-pan-ch="${s.index}" title="Pan" />`
        : "";
      return `
        <div class="strip">
          <div class="strip-scribble" style="background:${hex}" data-rename-kind="${kind}" data-idx="${s.index}">
            <span class="strip-idx">${s.index}</span>
            <span class="strip-name">${escapeHtml(s.name)}</span>
          </div>
          ${linkHtml}
          <button class="mute-btn ${s.on ? "is-on" : "is-muted"}" data-mute-kind="${kind}" data-idx="${s.index}" data-on="${s.on}">
            ${s.on ? "ON" : "MUTED"}
          </button>
          <div class="fader-zone">
            <div class="meter-track"><div class="meter-fill" style="height:${meterPct}%"></div></div>
            <input type="range" class="fader-slider" min="0" max="1" step="0.005" value="${s.fader}" data-fader-kind="${kind}" data-idx="${s.index}" />
          </div>
          ${panHtml}
          <div class="db-readout">${faderToDbLabel(s.fader)}</div>
        </div>
      `;
    }).join("");

    grid.querySelectorAll("[data-fader-kind]").forEach((el) => {
      el.addEventListener("input", (ev) => {
        const val = parseFloat(ev.target.value);
        ev.target.closest(".strip").querySelector(".db-readout").textContent = faderToDbLabel(val);
        api("/api/mixer/fader", "POST", { kind: el.dataset.faderKind, index: Number(el.dataset.idx), value: val }).catch(() => {});
      });
    });

    grid.querySelectorAll("[data-mute-kind]").forEach((el) => {
      el.addEventListener("click", () => {
        const nextOn = el.dataset.on !== "true";
        api("/api/mixer/mute", "POST", { kind: el.dataset.muteKind, index: Number(el.dataset.idx), on: nextOn }).catch(() => {});
      });
    });

    grid.querySelectorAll("[data-pan-ch]").forEach((el) => {
      el.addEventListener("input", (ev) => {
        api("/api/mixer/pan", "POST", { index: Number(el.dataset.panCh), pan: parseFloat(ev.target.value) }).catch(() => {});
      });
    });

    grid.querySelectorAll("[data-rename-kind]").forEach((el) => {
      el.addEventListener("click", () => {
        const k = el.dataset.renameKind;
        const idx = Number(el.dataset.idx);
        const list = k === "ch" ? mixerState.channels
          : k === "auxin" ? mixerState.auxins
          : k === "bus" ? mixerState.buses
          : k === "mtx" ? (mixerState.matrices || [])
          : mixerState.dcas;
        const cur = list[idx - 1];
        if (!cur) return;
        const nextName = prompt(`Rename ${k.toUpperCase()} ${idx}:`, cur.name);
        if (nextName !== null) {
          api("/api/mixer/channel-config", "POST", { kind: k, index: idx, name: nextName }).catch(() => {});
        }
      });
    });

    // Master strip
    const ms = $("masterStrip");
    ms.innerHTML = `
      <div class="strip-scribble" style="background:#f59e0b">
        <span class="strip-idx">LR</span>
        <span class="strip-name">MAIN LR</span>
      </div>
      <button class="mute-btn ${mixerState.main_on ? "is-on" : "is-muted"}" id="mainMuteBtn">
        ${mixerState.main_on ? "ON" : "MUTED"}
      </button>
      <div class="fader-zone">
        <input type="range" class="fader-slider" min="0" max="1" step="0.005" value="${mixerState.main_fader}" id="mainFaderSlider" />
      </div>
      <div class="db-readout" id="mainDbLabel">${faderToDbLabel(mixerState.main_fader)}</div>
    `;
    $("mainFaderSlider").addEventListener("input", (ev) => {
      const val = parseFloat(ev.target.value);
      $("mainDbLabel").textContent = faderToDbLabel(val);
      api("/api/mixer/fader", "POST", { kind: "main", index: 1, value: val }).catch(() => {});
    });
    $("mainMuteBtn").addEventListener("click", () => {
      api("/api/mixer/mute", "POST", { kind: "main", index: 1, on: !mixerState.main_on }).catch(() => {});
    });
  }

  // --- Virtual Soundcheck ---
  async function captureVsc(refresh) {
    try {
      const includeAux = $("vscIncludeAuxCard").checked;
      const preserveUout = $("vscPreserveUout").checked;
      const resp = await api("/api/vsc/capture", "POST", {
        refresh,
        include_aux_card: includeAux,
        preserve_card_uout: preserveUout
      });
      applyVscCaptureResponse(resp);
      appendDrawerLog(`Captured ${vscChannels.length} channels (${resp.session_options.sample_rate_hz} Hz).`);
    } catch (e) {
      appendDrawerLog(`VSC capture error: ${e.message}`);
    }
  }

  function applyVscCaptureResponse(resp) {
    updateMixerState(resp.state);
    vscChannels = resp.session_options.selected_channels;
    $("vscSampleRateSelect").value = String(resp.session_options.sample_rate_hz);
    $("vscSampleRateLabel").textContent = `${resp.session_options.sample_rate_hz} Hz`;
    $("vscIncludeEmpty").checked = !!resp.session_options.include_empty_channels;
    renderVscChannelTable();
    renderVscRoutingDiff(resp.routing_plan);
  }

  $("vscSceneFile").addEventListener("change", (ev) => {
    const file = ev.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = async () => {
      try {
        const resp = await api("/api/vsc/import-scene", "POST", {
          scene_content: String(reader.result || ""),
          include_aux_card: $("vscIncludeAuxCard").checked,
          preserve_card_uout: $("vscPreserveUout").checked
        });
        applyVscCaptureResponse(resp);
        appendDrawerLog(`Imported .scn file '${file.name}' (${resp.session_options.sample_rate_hz} Hz).`);
      } catch (e) {
        appendDrawerLog(`Scene import error: ${e.message}`);
      }
    };
    reader.readAsText(file);
  });

  function renderVscChannelTable() {
    const tbody = $("vscChannelTable").querySelector("tbody");
    tbody.innerHTML = vscChannels.map((c, idx) => `
      <tr>
        <td><input type="checkbox" data-vsc-inc="${idx}" ${c.include ? "checked" : ""} /></td>
        <td><strong>${String(c.channel).padStart(2, "0")}</strong></td>
        <td><input type="text" data-vsc-name="${idx}" value="${escapeHtml(c.name)}" /></td>
        <td><span class="color-dot" style="background:${colorHex(c.color)}" data-vsc-col="${idx}" title="Click to cycle X32 color"></span></td>
        <td><input type="checkbox" data-vsc-st="${idx}" ${c.linked_stereo ? "checked" : ""} /></td>
        <td><input type="number" min="1" max="32" data-vsc-in="${idx}" value="${c.card_in}" style="width:60px" /></td>
        <td><input type="number" min="1" max="32" data-vsc-out="${idx}" value="${c.card_out}" style="width:60px" /></td>
      </tr>
    `).join("");

    tbody.querySelectorAll("[data-vsc-inc]").forEach((el) => el.addEventListener("change", () => {
      vscChannels[el.dataset.vscInc].include = el.checked;
      $("vscActiveCount").textContent = String(vscChannels.filter((x) => x.include).length);
    }));
    tbody.querySelectorAll("[data-vsc-name]").forEach((el) => el.addEventListener("input", () => {
      vscChannels[el.dataset.vscName].name = el.value;
    }));
    tbody.querySelectorAll("[data-vsc-st]").forEach((el) => el.addEventListener("change", () => {
      vscChannels[el.dataset.vscSt].linked_stereo = el.checked;
    }));
    tbody.querySelectorAll("[data-vsc-in]").forEach((el) => el.addEventListener("change", () => {
      vscChannels[el.dataset.vscIn].card_in = Number(el.value) || 1;
    }));
    tbody.querySelectorAll("[data-vsc-out]").forEach((el) => el.addEventListener("change", () => {
      vscChannels[el.dataset.vscOut].card_out = Number(el.value) || 1;
    }));
    tbody.querySelectorAll("[data-vsc-col]").forEach((el) => el.addEventListener("click", () => {
      const i = Number(el.dataset.vscCol);
      vscChannels[i].color = (vscChannels[i].color + 1) % 8;
      el.style.background = colorHex(vscChannels[i].color);
    }));
    $("vscActiveCount").textContent = String(vscChannels.filter((x) => x.include).length);
  }

  function renderVscRoutingDiff(plan) {
    const tbody = $("vscRoutingDiffTable").querySelector("tbody");
    tbody.innerHTML = (plan.changes || []).map((c) => `
      <tr>
        <td>${escapeHtml(c.label)}</td>
        <td class="muted">${escapeHtml(c.before_label)}</td>
        <td><strong>${escapeHtml(c.after_label)}</strong></td>
      </tr>
    `).join("");
    const warnBox = $("vscWarningsBox");
    const warns = plan.warnings || [];
    if (warns.length) {
      warnBox.classList.remove("hidden");
      warnBox.innerHTML = warns.map((w) => `<div>&#9888; ${escapeHtml(w)}</div>`).join("");
    } else {
      warnBox.classList.add("hidden");
      warnBox.innerHTML = "";
    }
  }

  async function toggleVscMode(playMode) {
    try {
      const res = await api("/api/vsc/mode", "POST", { mode: playMode ? "play" : "rec" });
      appendDrawerLog(`Routswitch -> ${res.play_mode ? "PLAY (Virtual Soundcheck)" : "REC (Live)"}`);
    } catch (e) {
      appendDrawerLog(`Mode switch failed: ${e.message}`);
    }
  }

  $("vscCaptureLiveBtn").addEventListener("click", () => captureVsc(true));
  $("vscCaptureCachedBtn").addEventListener("click", () => captureVsc(false));
  $("vscModeRecBtn").addEventListener("click", () => toggleVscMode(false));
  $("vscModePlayBtn").addEventListener("click", () => toggleVscMode(true));

  $("vscApplyRoutingBtn").addEventListener("click", async () => {
    try {
      const res = await api("/api/vsc/apply-routing", "POST", {
        include_aux_card: $("vscIncludeAuxCard").checked,
        preserve_card_uout: $("vscPreserveUout").checked,
        backup_path: $("vscBackupPath").value.trim(),
        switch_to_play: $("vscSwitchOnApply").checked
      });
      renderVscRoutingDiff(res.plan);
      appendDrawerLog(`Applied ${res.applied_changes} VSC routing changes (backup saved to ${res.backup_path || "none"}).`);
    } catch (e) {
      appendDrawerLog(`Apply VSC routing failed: ${e.message}`);
    }
  });

  $("vscRestoreRoutingBtn").addEventListener("click", async () => {
    try {
      const res = await api("/api/vsc/restore-backup", "POST", {
        path: $("vscBackupPath").value.trim() || "x32_pre_vsc_backup.json"
      });
      appendDrawerLog(`Restored ${res.restored} routing entries from ${res.path}`);
    } catch (e) {
      appendDrawerLog(`Restore routing failed: ${e.message}`);
    }
  });

  $("vscTemplateFile").addEventListener("change", (ev) => {
    const file = ev.target.files[0];
    if (!file) { uploadedTemplateContent = null; return; }
    const reader = new FileReader();
    reader.onload = () => { uploadedTemplateContent = String(reader.result || ""); };
    reader.readAsText(file);
  });

  function currentSessionOptions() {
    return {
      project_name: $("vscProjectName").value.trim() || "X32_Virtual_Soundcheck",
      sample_rate_hz: Number($("vscSampleRateSelect").value) || 48000,
      record_path: $("vscRecordPath").value.trim() || "Audio",
      merge_stereo_links: $("vscMergeStereo").checked,
      arm_tracks_for_recording: $("vscArmTracks").checked,
      disable_master_send: $("vscDisableMaster").checked,
      include_empty_channels: $("vscIncludeEmpty").checked,
      deduplicate_shared_sources: $("vscDedupSources").checked,
      digital_twin_mode: $("vscDigitalTwin").checked,
      append_x32_reaper_tags: $("vscAppendTags").checked,
      include_dca_tracks: $("vscIncludeDcas").checked,
      include_aux_tracks: $("vscIncludeAux").checked,
      include_bus_tracks: $("vscIncludeBuses").checked,
      include_matrix_tracks: $("vscIncludeMatrices").checked,
      selected_channels: vscChannels,
      dca_names: (mixerState?.dcas || []).map((d) => d.name),
      aux_names: (mixerState?.auxins || []).map((a) => a.name),
      bus_names: (mixerState?.buses || []).map((b) => b.name),
      matrix_names: (mixerState?.matrices || []).map((m) => m.name),
    };
  }

  $("vscGenerateBtn").addEventListener("click", async () => {
    try {
      const body = {
        session_options: currentSessionOptions(),
        ini_options: {
          x32_ip: mixerState?.ip || $("ipInput").value.trim() || "127.0.0.1",
          reaper_ip: $("reaperIpInput").value.trim() || "127.0.0.1",
          reaper_send_port: Number($("reaperSendPortInput").value) || 8000,
          reaper_recv_port: Number($("reaperRecvPortInput").value) || 9000,
          bank_size: Number($("reaperBankSizeInput").value) || 32
        },
        template_content: uploadedTemplateContent,
        template_path: $("vscTemplatePath").value.trim() || null,
        output_rpp_path: $("vscOutputRppPath").value.trim() || null,
        output_ini_path: $("vscOutputIniPath").value.trim() || null
      };
      const res = await api("/api/vsc/generate-rpp", "POST", body);
      lastGeneratedRpp = res.rpp_content;
      lastGeneratedIni = res.ini_content;
      $("vscRppPreview").value = res.rpp_content;
      $("vscDownloadRppBtn").disabled = false;
      $("vscDownloadIniBtn").disabled = false;
      const savedMsg = [
        res.saved_rpp_path ? `Saved .rpp -> ${res.saved_rpp_path}` : null,
        res.saved_ini_path ? `Saved .ini -> ${res.saved_ini_path}` : null
      ].filter(Boolean).join(" | ");
      $("vscGenStatus").textContent = `Generated ${res.track_count} REAPER channel tracks. ${savedMsg}`;
      appendDrawerLog($("vscGenStatus").textContent);
    } catch (e) {
      $("vscGenStatus").textContent = `Error: ${e.message}`;
    }
  });

  $("vscDownloadRppBtn").addEventListener("click", () => {
    const name = ($("vscProjectName").value.trim() || "X32_Virtual_Soundcheck") + ".rpp";
    downloadTextFile(name, lastGeneratedRpp);
  });
  $("vscDownloadIniBtn").addEventListener("click", () => {
    downloadTextFile(".X32Reaper.ini", lastGeneratedIni);
  });

  $("reaperPushTracksBtn").addEventListener("click", async () => {
    try {
      const res = await api("/api/vsc/reaper-osc", "POST", {
        reaper_ip: $("reaperIpInput").value.trim() || "127.0.0.1",
        reaper_port: Number($("reaperSendPortInput").value) || 8000,
        session_options: currentSessionOptions()
      });
      $("reaperStatusLine").textContent = `Pushed ${res.messages_sent} OSC messages (${res.tracks} tracks) to REAPER.`;
    } catch (e) {
      $("reaperStatusLine").textContent = `REAPER OSC error: ${e.message}`;
    }
  });

  document.querySelectorAll("[data-reaper-act]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      try {
        const res = await api("/api/vsc/reaper-osc", "POST", {
          reaper_ip: $("reaperIpInput").value.trim() || "127.0.0.1",
          reaper_port: Number($("reaperSendPortInput").value) || 8000,
          action: btn.dataset.reaperAct
        });
        $("reaperStatusLine").textContent = `Sent REAPER action '${res.action}'.`;
      } catch (e) {
        $("reaperStatusLine").textContent = `REAPER action error: ${e.message}`;
      }
    });
  });

  $("startReaperBridgeBtn").addEventListener("click", async () => {
    try {
      const cfg = $("vscOutputIniPath").value.trim() || ".X32Reaper.ini";
      const job = await api("/api/tools/run", "POST", {
        tool_id: "x32-reaper",
        params: { config: cfg, bind_ip: "127.0.0.1" }
      });
      $("reaperStatusLine").textContent = `Started x32-reaper bridge (${job.job_id}).`;
    } catch (e) {
      $("reaperStatusLine").textContent = `Bridge launch error: ${e.message}`;
    }
  });

  // --- Routing Matrix ---
  $("routingRefreshBtn").addEventListener("click", () => $("refreshBtn").click());

  function renderRoutingMatrix() {
    if (!mixerState) return;
    const r = mixerState.routing;
    const blocks = ["1-8", "9-16", "17-24", "25-32"];
    const aesBlocks = ["1-8", "9-16", "17-24", "25-32", "33-40", "41-48"];
    const outBlocks = ["1-4", "5-8", "9-12", "13-16"];

    const selectHtml = (path, cur, opts) => `
      <div class="form-row">
        <label>${path}</label>
        <select data-route-path="${path}">
          ${opts.map(([v, lbl]) => `<option value="${v}" ${v === cur ? "selected" : ""}>${lbl}</option>`).join("")}
        </select>
      </div>
    `;

    $("routingGrid").innerHTML = `
      <div class="routing-section">
        <h4>Global Routing Switch</h4>
        ${selectHtml("/config/routing/routswitch", r.routswitch, [[0, "REC (Live Inputs)"], [1, "PLAY (Virtual Soundcheck)"]])}
      </div>
      <div class="routing-section">
        <h4>Home / Input Routing (IN)</h4>
        ${blocks.map((b, i) => selectHtml(`/config/routing/IN/${b}`, r.in_blocks[i], IN_OPTIONS)).join("")}
        ${selectHtml("/config/routing/IN/AUX", r.in_aux, AUX_OPTIONS)}
      </div>
      <div class="routing-section">
        <h4>Virtual Soundcheck Routing (PLAY)</h4>
        ${blocks.map((b, i) => selectHtml(`/config/routing/PLAY/${b}`, r.play_blocks[i], IN_OPTIONS)).join("")}
        ${selectHtml("/config/routing/PLAY/AUX", r.play_aux, AUX_OPTIONS)}
      </div>
      <div class="routing-section">
        <h4>Card Out (USB / X-Live)</h4>
        ${blocks.map((b, i) => selectHtml(`/config/routing/CARD/${b}`, r.card_blocks[i], CARD_OPTIONS)).join("")}
      </div>
      <div class="routing-section">
        <h4>Local XLR Outputs (OUT 1–16)</h4>
        ${outBlocks.map((b, i) => selectHtml(`/config/routing/OUT/${b}`, r.out_blocks[i], OUT_OPTIONS)).join("")}
      </div>
      <div class="routing-section">
        <h4>AES50-A Outputs</h4>
        ${aesBlocks.map((b, i) => selectHtml(`/config/routing/AES50A/${b}`, r.aes50a_blocks[i], CARD_OPTIONS)).join("")}
      </div>
      <div class="routing-section">
        <h4>AES50-B Outputs</h4>
        ${aesBlocks.map((b, i) => selectHtml(`/config/routing/AES50B/${b}`, r.aes50b_blocks[i], CARD_OPTIONS)).join("")}
      </div>
    `;

    $("routingGrid").querySelectorAll("[data-route-path]").forEach((sel) => {
      sel.addEventListener("change", async () => {
        try {
          await api("/api/mixer/routing", "POST", { path: sel.dataset.routePath, value: Number(sel.value) });
          appendDrawerLog(`Routing ${sel.dataset.routePath} = ${sel.value}`);
        } catch (e) {
          appendDrawerLog(`Routing error: ${e.message}`);
        }
      });
    });
  }

  // --- All Tools Catalog ---
  async function loadTools() {
    const data = await api("/api/tools");
    toolsCatalog = data.tools || [];
    jobsList = data.jobs || [];
    $("toolCount").textContent = String(toolsCatalog.length);
    renderCategoryPills();
    renderToolsGrid();
    renderJobs();
  }

  function renderCategoryPills() {
    const cats = ["All", ...new Set(toolsCatalog.map((t) => t.category))];
    $("categoryPills").innerHTML = cats.map((c) => `
      <button class="bank-btn ${c === activeCategory ? "active" : ""}" data-cat="${escapeHtml(c)}">${escapeHtml(c)}</button>
    `).join("");
    $("categoryPills").querySelectorAll("[data-cat]").forEach((b) => {
      b.addEventListener("click", () => {
        activeCategory = b.dataset.cat;
        renderCategoryPills();
        renderToolsGrid();
      });
    });
  }

  function renderToolsGrid() {
    const q = ($("toolSearchInput").value || "").toLowerCase();
    const filtered = toolsCatalog.filter((t) => {
      const catOk = activeCategory === "All" || t.category === activeCategory;
      const qOk = !q || t.name.toLowerCase().includes(q) || t.id.toLowerCase().includes(q) || t.description.toLowerCase().includes(q);
      return catOk && qOk;
    });
    $("toolsGrid").innerHTML = filtered.map((t) => `
      <div class="tool-card ${t.id === selectedToolId ? "selected" : ""}" data-tool-id="${t.id}">
        <div class="tool-card-top">
          <h4>${escapeHtml(t.name)}</h4>
          <span class="mode-tag">${t.mode}</span>
        </div>
        <div class="muted">${escapeHtml(t.id)} &bull; ${escapeHtml(t.category)}</div>
        <div style="font-size:12px;color:#cbd5e1">${escapeHtml(t.description)}</div>
      </div>
    `).join("");
    $("toolsGrid").querySelectorAll("[data-tool-id]").forEach((card) => {
      card.addEventListener("click", () => {
        selectedToolId = card.dataset.toolId;
        renderToolsGrid();
        renderToolRunner();
      });
    });
  }

  $("toolSearchInput").addEventListener("input", renderToolsGrid);

  function renderToolRunner() {
    const tool = toolsCatalog.find((t) => t.id === selectedToolId);
    const panel = $("toolRunnerPanel");
    if (!tool) return;

    const currentIp = mixerState?.ip || $("ipInput").value.trim() || "127.0.0.1:10023";
    const fieldsHtml = tool.fields.map((f) => {
      const val = f.field_type === "ip" && currentIp ? currentIp : (f.default_value || "");
      if (f.field_type === "bool") {
        const checked = val === "true" ? "checked" : "";
        return `<label class="checkbox-inline" style="margin:6px 0"><input type="checkbox" data-field="${f.name}" ${checked} /> ${escapeHtml(f.label)} <span class="muted">(${f.flag})</span></label>`;
      }
      if (f.field_type === "select") {
        return `
          <div class="form-row">
            <label>${escapeHtml(f.label)} (${f.flag})</label>
            <select data-field="${f.name}">
              ${f.options.map((o) => `<option value="${o}" ${o === val ? "selected" : ""}>${o}</option>`).join("")}
            </select>
          </div>
        `;
      }
      return `
        <div class="form-row">
          <label>${escapeHtml(f.label)} ${f.flag ? `(${f.flag})` : "(positional)"} ${f.required ? "*" : ""}</label>
          <input type="${f.field_type === "number" ? "number" : "text"}" data-field="${f.name}" value="${escapeHtml(val)}" placeholder="${escapeHtml(f.placeholder)}" />
        </div>
      `;
    }).join("");

    panel.innerHTML = `
      <h3>${escapeHtml(tool.name)}</h3>
      <p class="muted">${escapeHtml(tool.id)} (${tool.mode})</p>
      <p style="font-size:12px">${escapeHtml(tool.description)}</p>
      ${fieldsHtml}
      <div class="btn-row" style="margin-top:10px">
        <button class="btn btn-primary" id="runSelectedToolBtn">Launch Tool</button>
      </div>
      <div id="toolRunStatus" class="status-line"></div>
    `;

    $("runSelectedToolBtn").addEventListener("click", async () => {
      const params = {};
      panel.querySelectorAll("[data-field]").forEach((el) => {
        params[el.dataset.field] = el.type === "checkbox" ? el.checked : el.value;
      });
      try {
        const job = await api("/api/tools/run", "POST", { tool_id: tool.id, params });
        $("toolRunStatus").textContent = `${job.job_id}: ${job.status}`;
        appendDrawerLog(`Launched ${tool.id} (${job.job_id}): ${job.command_line}`);
        await refreshJobs();
      } catch (e) {
        $("toolRunStatus").textContent = `Error: ${e.message}`;
      }
    });
  }

  // --- OSC & Jobs ---
  $("oscSendBtn").addEventListener("click", async () => {
    const path = $("oscPathInput").value.trim();
    const argType = $("oscArgType").value;
    const val = $("oscArgValue").value;
    const args = argType === "none" ? [] : [{ type: argType, value: val }];
    try {
      await api("/api/osc", "POST", { path, args });
      appendDrawerLog(`OSC -> ${path}`);
    } catch (e) {
      appendDrawerLog(`OSC error: ${e.message}`);
    }
  });

  $("refreshJobsBtn").addEventListener("click", refreshJobs);

  async function refreshJobs() {
    try {
      jobsList = await api("/api/tools/jobs");
      renderJobs();
    } catch (_) {}
  }

  function renderJobs() {
    $("jobsList").innerHTML = jobsList.map((j) => `
      <div class="job-row">
        <div>
          <strong>${escapeHtml(j.tool_id)}</strong> <span class="muted">(${j.job_id} &bull; ${j.status})</span>
        </div>
        <div class="btn-row">
          <button class="btn btn-sm" data-view-job="${j.job_id}">Logs</button>
          ${j.status === "running" ? `<button class="btn btn-sm btn-danger" data-stop-job="${j.job_id}">Stop</button>` : ""}
        </div>
      </div>
    `).join("") || `<div class="muted">No tools launched yet.</div>`;

    $("jobsList").querySelectorAll("[data-stop-job]").forEach((b) => {
      b.addEventListener("click", async () => {
        await api("/api/tools/stop", "POST", { job_id: b.dataset.stopJob });
        refreshJobs();
      });
    });
    $("jobsList").querySelectorAll("[data-view-job]").forEach((b) => {
      b.addEventListener("click", () => {
        const job = jobsList.find((x) => x.job_id === b.dataset.viewJob);
        if (job) $("jobLogOutput").textContent = (job.logs || []).join("\n") || job.command_line;
      });
    });
  }

  function renderOscLog(entries) {
    const tbody = $("oscLogTable").querySelector("tbody");
    tbody.innerHTML = (entries || []).slice().reverse().slice(0, 60).map((e) => `
      <tr>
        <td>${e.direction === "out" ? "OUT" : "IN"}</td>
        <td><code>${escapeHtml(e.path)}</code></td>
        <td><code>${escapeHtml(e.args)}</code></td>
      </tr>
    `).join("");
  }

  function updateMixerState(snap) {
    const firstLoad = !mixerState;
    mixerState = snap;
    $("connBadge").textContent = snap.connected ? snap.connection_state : "Disconnected";
    $("connBadge").className = `pill ${snap.connected ? "pill-on" : "pill-off"}`;
    $("modelSubtitle").textContent = snap.connected
      ? `${snap.model} @ ${snap.ip} (${snap.transport.toUpperCase()}) • ${snap.sample_rate_hz} Hz`
      : "Disconnected — Connect or Start Local Emulator";

    const isPlay = snap.routing && snap.routing.routswitch === 1;
    const modePill = $("globalModePill");
    modePill.textContent = isPlay ? "MODE: PLAY (VSC)" : "MODE: REC (LIVE)";
    modePill.className = `pill ${isPlay ? "pill-play" : "pill-rec"}`;
    $("vscModeRecBtn").classList.toggle("active", !isPlay);
    $("vscModePlayBtn").classList.toggle("active", isPlay);
    $("vscCurrentSwitch").textContent = isPlay ? "PLAY (Virtual Soundcheck)" : "REC (Live Inputs)";
    $("vscSampleRateLabel").textContent = `${snap.sample_rate_hz} Hz`;

    if (firstLoad || !vscChannels.length) {
      vscChannels = snap.channels.map((c) => ({
        channel: c.index,
        name: c.name,
        color: c.color,
        icon: c.icon || 1,
        linked_stereo: c.linked_stereo,
        include: true,
        card_in: c.index,
        card_out: c.index
      }));
      renderVscChannelTable();
    }

    renderMixer();
    renderRoutingMatrix();
    if (snap.recent_osc && snap.recent_osc.length) {
      renderOscLog(snap.recent_osc);
    }
  }

  function startSse() {
    const es = new EventSource("/api/events");
    es.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data);
        if (msg.type === "State") {
          updateMixerState(msg.data);
        } else if (msg.type === "Osc") {
          if (mixerState) {
            mixerState.recent_osc = [...(mixerState.recent_osc || []), msg.data].slice(-80);
            renderOscLog(mixerState.recent_osc);
          }
        } else if (msg.type === "JobLog") {
          appendDrawerLog(`[${msg.data.job_id}] ${msg.data.line}`);
        } else if (msg.type === "JobStatus") {
          refreshJobs();
        } else if (msg.type === "VscStatus") {
          appendDrawerLog(msg.data.message);
        } else if (msg.type === "StudioStatus") {
          updateStudioStatus(msg.data);
        }
      } catch (_) {}
    };
  }

  // --- Studio Mode: Digital Split, Event Recorder & Hardware Deck ---
  let studioStatus = null;

  async function loadStudioStatus() {
    try {
      const s = await api("/api/studio/status");
      updateStudioStatus(s);
      await loadStudioRecordings();
    } catch (_) {}
  }

  async function loadStudioRecordings() {
    try {
      const list = await api("/api/studio/recordings");
      const sel = $("studioRecordingsSelect");
      sel.innerHTML = (list || []).map((r) =>
        `<option value="${escapeHtml(r.path)}">${escapeHtml(r.name)} (${r.events} ev, ${r.markers} mkr)</option>`
      ).join("") || `<option value="">No recordings yet</option>`;
    } catch (_) {}
  }

  function updateStudioStatus(s) {
    studioStatus = s;
    const isDaw = s.mode === "daw";
    $("studioModeLiveBtn").classList.toggle("active", !isDaw);
    $("studioModeDawBtn").classList.toggle("active", isDaw);
    $("studioSplitBadge").textContent = s.digital_split_applied ? "Active" : "Not applied";
    $("studioPopGuardBadge").textContent = s.pop_guard_active ? "Muting..." : `${s.profile.pop_guard_mute_ms} ms`;
    $("studioBridgeBadge").textContent = s.reaper_bridge_job_id || "Off";
    $("studioLastAction").textContent = s.last_action || "Ready";
    $("studioBankLabel").textContent = s.control_bank || "C";

    if (s.recorder) {
      $("recStateLabel").textContent = s.recorder.state;
      $("recEventCount").textContent = String(s.recorder.event_count);
      $("recMarkerCount").textContent = String(s.recorder.marker_count);
    }

    if (s.profile) {
      $("profMixerIp").value = s.profile.mixer_ip || "auto";
      $("profMirrorAes50").value = s.profile.split_mirror_aes50 || "none";
      $("profControlBank").value = s.profile.control_bank || "C";
      $("profPopGuardMs").value = String(s.profile.pop_guard_mute_ms ?? 60);
      $("profRecDir").value = s.profile.recordings_dir || "recordings";
      $("profReaperSession").value = s.profile.reaper_session_path || "";
      $("profAutoConnect").checked = !!s.profile.auto_connect_on_boot;
      $("profAutoSplit").checked = !!s.profile.auto_apply_digital_split;
      $("profEnableBtns").checked = !!s.profile.enable_console_buttons;
      $("profSyncReplay").checked = !!s.profile.sync_event_replay_with_daw;
      $("profRestoreLive").checked = !!s.profile.restore_live_snapshot_on_return;
      $("profBridgeInDaw").checked = !!s.profile.launch_daw_bridge_in_daw_mode;
      $("profAutoLaunchReaper").checked = !!s.profile.auto_launch_reaper;
      $("profPreserveUout").checked = s.profile.preserve_card_uout !== false;
      $("profAutoGenRpp").checked = s.profile.auto_generate_rpp_on_connect !== false;
    }

    const deck = $("studioHwDeck");
    deck.innerHTML = (s.buttons || []).map((b, i) => `
      <button class="hw-btn ${b.lit ? "is-lit" : ""}" data-hw-offset="${i}">
        <span class="hw-led"></span>
        <span>${escapeHtml(b.label)}</span>
      </button>
    `).join("");
    deck.querySelectorAll("[data-hw-offset]").forEach((btn) => {
      btn.addEventListener("click", async () => {
        try {
          const updated = await api("/api/studio/button", "POST", { offset: Number(btn.dataset.hwOffset) });
          updateStudioStatus(updated);
        } catch (e) {
          appendDrawerLog(`Button trigger error: ${e.message}`);
        }
      });
    });
  }

  $("studioModeLiveBtn").addEventListener("click", async () => {
    try { updateStudioStatus(await api("/api/studio/mode", "POST", { mode: "live" })); }
    catch (e) { appendDrawerLog(`Mode switch error: ${e.message}`); }
  });
  $("studioModeDawBtn").addEventListener("click", async () => {
    try { updateStudioStatus(await api("/api/studio/mode", "POST", { mode: "daw" })); }
    catch (e) { appendDrawerLog(`Mode switch error: ${e.message}`); }
  });
  $("studioApplySplitBtn").addEventListener("click", async () => {
    try {
      const res = await api("/api/studio/apply-split", "POST", {});
      updateStudioStatus(res.status);
    } catch (e) {
      appendDrawerLog(`Digital split error: ${e.message}`);
    }
  });
  $("studioRecStartBtn").addEventListener("click", async () => {
    try {
      const label = $("recSessionLabel").value.trim() || "LiveSession";
      updateStudioStatus(await api("/api/studio/record/start", "POST", { label }));
      await loadStudioRecordings();
    } catch (e) {
      appendDrawerLog(`Record start error: ${e.message}`);
    }
  });
  $("studioPlayBtn").addEventListener("click", async () => {
    try {
      const file = $("studioRecordingsSelect").value || null;
      updateStudioStatus(await api("/api/studio/play", "POST", { file }));
    } catch (e) {
      appendDrawerLog(`Play error: ${e.message}`);
    }
  });
  $("studioStopBtn").addEventListener("click", async () => {
    try {
      updateStudioStatus(await api("/api/studio/stop", "POST", {}));
      await loadStudioRecordings();
    } catch (e) {
      appendDrawerLog(`Stop error: ${e.message}`);
    }
  });
  $("studioMarkerBtn").addEventListener("click", async () => {
    try {
      const label = $("recMarkerLabel").value.trim() || null;
      updateStudioStatus(await api("/api/studio/marker", "POST", { label }));
    } catch (e) {
      appendDrawerLog(`Marker error: ${e.message}`);
    }
  });
  $("studioRefreshRecsBtn").addEventListener("click", loadStudioRecordings);
  $("studioProgramBtnsBtn").addEventListener("click", async () => {
    try { updateStudioStatus(await api("/api/studio/program-buttons", "POST", {})); }
    catch (e) { appendDrawerLog(`Program buttons error: ${e.message}`); }
  });
  $("studioSaveProfileBtn").addEventListener("click", async () => {
    const base = studioStatus?.profile || {};
    const payload = {
      ...base,
      mixer_ip: $("profMixerIp").value.trim() || "auto",
      split_mirror_aes50: $("profMirrorAes50").value,
      control_bank: $("profControlBank").value,
      pop_guard_mute_ms: Number($("profPopGuardMs").value) || 0,
      recordings_dir: $("profRecDir").value.trim() || "recordings",
      reaper_session_path: $("profReaperSession").value.trim(),
      auto_connect_on_boot: $("profAutoConnect").checked,
      auto_apply_digital_split: $("profAutoSplit").checked,
      enable_console_buttons: $("profEnableBtns").checked,
      sync_event_replay_with_daw: $("profSyncReplay").checked,
      restore_live_snapshot_on_return: $("profRestoreLive").checked,
      launch_daw_bridge_in_daw_mode: $("profBridgeInDaw").checked,
      auto_launch_reaper: $("profAutoLaunchReaper").checked,
      preserve_card_uout: $("profPreserveUout").checked,
      auto_generate_rpp_on_connect: $("profAutoGenRpp").checked,
      reaper_ip: $("reaperIpInput").value.trim() || "127.0.0.1",
      reaper_send_port: Number($("reaperSendPortInput").value) || 8000,
      reaper_recv_port: Number($("reaperRecvPortInput").value) || 9000,
    };
    try {
      updateStudioStatus(await api("/api/studio/profile", "POST", payload));
      appendDrawerLog("Saved Turn-On & Go studio profile.");
    } catch (e) {
      appendDrawerLog(`Save profile error: ${e.message}`);
    }
  });

  function downloadTextFile(filename, content) {
    const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  function escapeHtml(s) {
    return String(s ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;");
  }

  (async function init() {
    const initial = await api("/api/state");
    updateMixerState(initial);
    await captureVsc(false);
    await loadTools();
    await loadStudioStatus();
    startSse();
  })();
})();
