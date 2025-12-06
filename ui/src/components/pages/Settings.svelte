<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import {
        settingsStore,
        setSettings,
        updateSetting,
        setSaving,
        clearErrors,
        isDirty,
        isBusy,
    } from "../../stores/settingsStore";
    import type { Settings } from "../../stores/settingsStore";
    import {
        themeStore,
        setTheme,
        setAccentColor,
        getThemeColors,
        getAccentColorValue,
        type Theme,
        type AccentColor,
    } from "../../stores/themeStore";
    import {
        languageStore,
        setLanguage,
        getSupportedLanguages,
        getLanguageName,
        type Language,
    } from "../../stores/languageStore";
    import Button from "../core/Button.svelte";
    import Input from "../core/Input.svelte";
    import Toast from "../core/Toast.svelte";

    let loading = $state(false);
    let toastMessage = $state("");
    let toastType = $state<"success" | "error" | "info" | "warning">("info");
    let showToast = $state(false);

    let settingsData = $state<any>({});
    let settingsErrors = $state<any>({});
    let dirtyState = $state(false);
    let busyState = $state(false);

    let currentTheme = $state<Theme>("dark");
    let currentAccentColor = $state<AccentColor>("red");
    let currentLanguage = $state<Language>("en");
    let supportedLanguages = $state<Language[]>([]);

    const unsubscribeSettings = settingsStore.subscribe((value) => {
        settingsData = value.data;
        settingsErrors = value.errors;
    });

    const unsubscribeDirty = isDirty.subscribe((value) => {
        dirtyState = value;
    });

    const unsubscribeBusy = isBusy.subscribe((value) => {
        busyState = value;
    });

    const unsubscribeTheme = themeStore.subscribe((value) => {
        currentTheme = value.theme;
        currentAccentColor = value.accentColor;
    });

    const unsubscribeLanguage = languageStore.subscribe((value) => {
        currentLanguage = value.currentLanguage;
    });

    onMount(() => {
        loadSettings();
        supportedLanguages = getSupportedLanguages();
        return () => {
            unsubscribeSettings();
            unsubscribeDirty();
            unsubscribeBusy();
            unsubscribeTheme();
            unsubscribeLanguage();
        };
    });

    async function loadSettings() {
        loading = true;
        try {
            const response = await invoke("get_settings");
            setSettings(response as Settings);
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            showToastMessage(errorMsg, "error");
            console.error("Failed to load settings:", err);
        } finally {
            loading = false;
        }
    }

    async function saveSettings() {
        setSaving(true);
        clearErrors();
        try {
            await invoke("save_settings", { settings: settingsData });
            setSettings(settingsData);
            showToastMessage("Настройки сохранены", "success");
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            showToastMessage(`Ошибка: ${errorMsg}`, "error");
            console.error("Failed to save settings:", err);
        } finally {
            setSaving(false);
        }
    }

    async function testQBittorrent() {
        try {
            const url = `${settingsData.qbittorrentUseHttps ? "https" : "http"}://${settingsData.qbittorrentHost}:${settingsData.qbittorrentPort}`;
            await invoke("test_qbittorrent", {
                url,
                username: settingsData.qbittorrentUsername || "",
                password: settingsData.qbittorrentPassword || "",
            });
            showToastMessage("Соединение успешно", "success");
        } catch (err) {
            const errorMsg = err instanceof Error ? err.message : String(err);
            showToastMessage(`Ошибка подключения: ${errorMsg}`, "error");
        }
    }

    async function pickLibraryPath() {
        try {
            const path = await invoke("pick_folder");
            updateSetting("libraryPath", path as string);
        } catch (err) {
            console.error("Failed to pick folder:", err);
        }
    }

    async function pickDownloadPath() {
        try {
            const path = await invoke("pick_folder");
            updateSetting("downloadPath", path as string);
        } catch (err) {
            console.error("Failed to pick folder:", err);
        }
    }

    function showToastMessage(
        message: string,
        type: "success" | "error" | "info" | "warning",
    ) {
        toastMessage = message;
        toastType = type;
        showToast = true;
        setTimeout(
            () => {
                showToast = false;
            },
            type === "error" ? 5000 : 3000,
        );
    }

    function handleInputChange(field: keyof Settings, value: any) {
        updateSetting(field, value);
    }

    function handleInputEvent(field: keyof Settings, e: Event) {
        const target = e.target as HTMLInputElement;
        if (
            field === "searchLimit" ||
            field === "searchTimeout" ||
            field === "maxConcurrentDownloads" ||
            field === "qbittorrentPort" ||
            field === "telegramDebounce" ||
            field === "emailSmtpPort" ||
            field === "proxyPort"
        ) {
            handleInputChange(field, parseInt(target.value));
        } else {
            handleInputChange(field, target.value);
        }
    }

    function handleLogLevelChange(e: Event) {
        const value = (e.target as HTMLSelectElement).value;
        if (
            value === "DEBUG" ||
            value === "INFO" ||
            value === "WARN" ||
            value === "ERROR"
        ) {
            handleInputChange("logLevel", value);
        }
    }

    function handleThemeChange(e: Event) {
        const value = (e.target as HTMLSelectElement).value as Theme;
        setTheme(value);
    }

    function handleAccentColorChange(e: Event) {
        const value = (e.target as HTMLSelectElement).value as AccentColor;
        setAccentColor(value);
    }

    async function handleLanguageChange(e: Event) {
        const value = (e.target as HTMLSelectElement).value as Language;
        await setLanguage(value);
    }
</script>

<div
    class="flex flex-col h-full bg-netflix-dark-bg overflow-hidden page-transition"
>
    {#if loading}
        <div class="flex items-center justify-center flex-1">
            <div class="text-center">
                <div
                    class="animate-spin rounded-full h-12 w-12 border-b-2 border-netflix-red mx-auto mb-4"
                ></div>
                <p class="text-white/70">Загрузка настроек...</p>
            </div>
        </div>
    {:else}
        <div class="flex-1 overflow-y-auto p-6">
            <div class="max-w-4xl mx-auto">
                <h1 class="text-3xl font-bold text-white mb-8">⚙️ Настройки</h1>

                <!-- API & Metadata Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        🔑 API & Метаданные
                    </h2>
                    <div class="space-y-4">
                        <Input
                            label="TMDB API Key"
                            type="password"
                            placeholder="Введите TMDB API ключ"
                            value={settingsData.tmdbApiKey || ""}
                            on:input={(e) => handleInputEvent("tmdbApiKey", e)}
                            error={settingsErrors.tmdbApiKey}
                        />
                        <Input
                            label="Kinopoisk Token"
                            type="password"
                            placeholder="Введите Kinopoisk токен"
                            value={settingsData.kinopoiskToken || ""}
                            on:input={(e) =>
                                handleInputEvent("kinopoiskToken", e)}
                            error={settingsErrors.kinopoiskToken}
                        />
                    </div>
                </div>

                <!-- qBittorrent Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        🧲 qBittorrent
                    </h2>
                    <div class="space-y-4">
                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                checked={settingsData.qbittorrentEnabled}
                                on:change={(e) =>
                                    handleInputChange(
                                        "qbittorrentEnabled",
                                        e.currentTarget.checked,
                                    )}
                                class="checkbox-styled"
                            />
                            <span class="text-white">Включить qBittorrent</span>
                        </label>

                        {#if settingsData.qbittorrentEnabled}
                            <Input
                                label="Host"
                                type="text"
                                placeholder="localhost"
                                value={settingsData.qbittorrentHost}
                                on:input={(e) =>
                                    handleInputEvent("qbittorrentHost", e)}
                                error={settingsErrors.qbittorrentHost}
                            />
                            <Input
                                label="Port"
                                type="number"
                                placeholder="5555"
                                value={String(settingsData.qbittorrentPort)}
                                on:input={(e) =>
                                    handleInputEvent("qbittorrentPort", e)}
                                error={settingsErrors.qbittorrentPort}
                            />
                            <Input
                                label="Username"
                                type="text"
                                placeholder="Имя пользователя"
                                value={settingsData.qbittorrentUsername || ""}
                                on:input={(e) =>
                                    handleInputEvent("qbittorrentUsername", e)}
                                error={settingsErrors.qbittorrentUsername}
                            />
                            <Input
                                label="Password"
                                type="password"
                                placeholder="Пароль"
                                value={settingsData.qbittorrentPassword || ""}
                                on:input={(e) =>
                                    handleInputEvent("qbittorrentPassword", e)}
                                error={settingsErrors.qbittorrentPassword}
                            />

                            <div class="flex space-x-4">
                                <label class="flex items-center space-x-2">
                                    <input
                                        type="checkbox"
                                        checked={settingsData.qbittorrentUseHttps}
                                        on:change={(e) =>
                                            handleInputChange(
                                                "qbittorrentUseHttps",
                                                e.currentTarget.checked,
                                            )}
                                        class="checkbox-styled"
                                    />
                                    <span class="text-white text-sm"
                                        >Использовать HTTPS</span
                                    >
                                </label>
                                <label class="flex items-center space-x-2">
                                    <input
                                        type="checkbox"
                                        checked={settingsData.qbittorrentVerifySsl}
                                        on:change={(e) =>
                                            handleInputChange(
                                                "qbittorrentVerifySsl",
                                                e.currentTarget.checked,
                                            )}
                                        class="checkbox-styled"
                                    />
                                    <span class="text-white text-sm"
                                        >Проверять SSL</span
                                    >
                                </label>
                            </div>

                            <Button
                                variant="secondary"
                                on:click={testQBittorrent}
                                disabled={busyState}
                            >
                                Проверить соединение
                            </Button>
                        {/if}
                    </div>
                </div>

                <!-- Filesystem Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        📁 Файловая система
                    </h2>
                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Путь библиотеки</label
                            >
                            <div class="flex space-x-2">
                                <Input
                                    type="text"
                                    placeholder="Выберите папку библиотеки"
                                    value={settingsData.libraryPath}
                                    on:input={(e) =>
                                        handleInputEvent("libraryPath", e)}
                                    error={settingsErrors.libraryPath}
                                    disabled={true}
                                />
                                <Button
                                    variant="secondary"
                                    on:click={pickLibraryPath}
                                >
                                    Обзор
                                </Button>
                            </div>
                        </div>

                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Путь загрузок</label
                            >
                            <div class="flex space-x-2">
                                <Input
                                    type="text"
                                    placeholder="Выберите папку загрузок"
                                    value={settingsData.downloadPath}
                                    on:input={(e) =>
                                        handleInputEvent("downloadPath", e)}
                                    error={settingsErrors.downloadPath}
                                    disabled={true}
                                />
                                <Button
                                    variant="secondary"
                                    on:click={pickDownloadPath}
                                >
                                    Обзор
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Notifications Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        🔔 Уведомления
                    </h2>
                    <div class="space-y-6">
                        <!-- Telegram -->
                        <div>
                            <label class="flex items-center space-x-3 mb-4">
                                <input
                                    type="checkbox"
                                    checked={settingsData.telegramEnabled}
                                    on:change={(e) =>
                                        handleInputChange(
                                            "telegramEnabled",
                                            e.currentTarget.checked,
                                        )}
                                    class="checkbox-styled"
                                />
                                <span class="text-white font-medium"
                                    >Telegram</span
                                >
                            </label>

                            {#if settingsData.telegramEnabled}
                                <div class="space-y-4 ml-8">
                                    <Input
                                        label="Bot Token"
                                        type="password"
                                        placeholder="Введите токен бота"
                                        value={settingsData.telegramBotToken ||
                                            ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "telegramBotToken",
                                                e,
                                            )}
                                        error={settingsErrors.telegramBotToken}
                                    />
                                    <Input
                                        label="Chat ID"
                                        type="text"
                                        placeholder="Введите ID чата"
                                        value={settingsData.telegramChatId ||
                                            ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "telegramChatId",
                                                e,
                                            )}
                                        error={settingsErrors.telegramChatId}
                                    />
                                    <Input
                                        label="Debounce (ms)"
                                        type="number"
                                        placeholder="1000"
                                        value={String(
                                            settingsData.telegramDebounce ||
                                                1000,
                                        )}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "telegramDebounce",
                                                e,
                                            )}
                                        error={settingsErrors.telegramDebounce}
                                    />
                                </div>
                            {/if}
                        </div>

                        <!-- Email -->
                        <div>
                            <label class="flex items-center space-x-3 mb-4">
                                <input
                                    type="checkbox"
                                    checked={settingsData.emailEnabled}
                                    on:change={(e) =>
                                        handleInputChange(
                                            "emailEnabled",
                                            e.currentTarget.checked,
                                        )}
                                    class="checkbox-styled"
                                />
                                <span class="text-white font-medium">Email</span
                                >
                            </label>

                            {#if settingsData.emailEnabled}
                                <div class="space-y-4 ml-8">
                                    <Input
                                        label="SMTP Host"
                                        type="text"
                                        placeholder="smtp.gmail.com"
                                        value={settingsData.emailSmtpHost || ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "emailSmtpHost",
                                                e,
                                            )}
                                        error={settingsErrors.emailSmtpHost}
                                    />
                                    <Input
                                        label="SMTP Port"
                                        type="number"
                                        placeholder="587"
                                        value={String(
                                            settingsData.emailSmtpPort || 587,
                                        )}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "emailSmtpPort",
                                                e,
                                            )}
                                        error={settingsErrors.emailSmtpPort}
                                    />
                                    <Input
                                        label="Username"
                                        type="text"
                                        placeholder="Имя пользователя"
                                        value={settingsData.emailUsername || ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "emailUsername",
                                                e,
                                            )}
                                        error={settingsErrors.emailUsername}
                                    />
                                    <Input
                                        label="Password"
                                        type="password"
                                        placeholder="Пароль"
                                        value={settingsData.emailPassword || ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "emailPassword",
                                                e,
                                            )}
                                        error={settingsErrors.emailPassword}
                                    />
                                    <Input
                                        label="From Email"
                                        type="email"
                                        placeholder="from@example.com"
                                        value={settingsData.emailFromAddress ||
                                            ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "emailFromAddress",
                                                e,
                                            )}
                                        error={settingsErrors.emailFromAddress}
                                    />
                                    <Input
                                        label="To Email"
                                        type="email"
                                        placeholder="to@example.com"
                                        value={settingsData.emailToAddress ||
                                            ""}
                                        on:input={(e) =>
                                            handleInputEvent(
                                                "emailToAddress",
                                                e,
                                            )}
                                        error={settingsErrors.emailToAddress}
                                    />
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>

                <!-- Application Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        🎛️ Приложение
                    </h2>
                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Уровень логирования</label
                            >
                            <select
                                value={settingsData.logLevel}
                                on:change={handleLogLevelChange}
                                class="select-styled"
                            >
                                <option value="DEBUG">DEBUG</option>
                                <option value="INFO">INFO</option>
                                <option value="WARN">WARN</option>
                                <option value="ERROR">ERROR</option>
                            </select>
                        </div>

                        <Input
                            label="Лимит поиска"
                            type="number"
                            placeholder="50"
                            value={String(settingsData.searchLimit)}
                            on:input={(e) => handleInputEvent("searchLimit", e)}
                            error={settingsErrors.searchLimit}
                        />

                        <Input
                            label="Timeout поиска (сек)"
                            type="number"
                            placeholder="30"
                            value={String(settingsData.searchTimeout)}
                            on:input={(e) =>
                                handleInputEvent("searchTimeout", e)}
                            error={settingsErrors.searchTimeout}
                        />

                        <Input
                            label="Макс. одновременных загрузок"
                            type="number"
                            placeholder="3"
                            value={String(settingsData.maxConcurrentDownloads)}
                            on:input={(e) =>
                                handleInputEvent("maxConcurrentDownloads", e)}
                            error={settingsErrors.maxConcurrentDownloads}
                        />

                        <Input
                            label="Включенные индексеры"
                            type="text"
                            placeholder="monna, другой_индексер"
                            value={settingsData.enabledIndexers || ""}
                            on:input={(e) =>
                                handleInputEvent("enabledIndexers", e)}
                            error={settingsErrors.enabledIndexers}
                        />

                        <label class="flex items-center space-x-3">
                            <input
                                type="checkbox"
                                checked={settingsData.debugMode}
                                on:change={(e) =>
                                    handleInputChange(
                                        "debugMode",
                                        e.currentTarget.checked,
                                    )}
                                class="checkbox-styled"
                            />
                            <span class="text-white">Режим отладки</span>
                        </label>
                    </div>
                </div>

                <!-- UI Preferences Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        🎨 Предпочтения UI
                    </h2>
                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Режим пагинации</label
                            >
                            <select
                                value={settingsData.paginationMode}
                                on:change={(e) =>
                                    handleInputChange(
                                        "paginationMode",
                                        (e.target as HTMLSelectElement).value,
                                    )}
                                class="select-styled"
                            >
                                <option value="infinite"
                                    >Бесконечная прокрутка</option
                                >
                                <option value="button"
                                    >Кнопка "Загрузить еще"</option
                                >
                            </select>
                            <p class="text-gray-400 text-xs mt-2">
                                Выберите, как загружать больше фильмов
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Theme & Appearance Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">
                        🎨 Тема и внешний вид
                    </h2>
                    <div class="space-y-6">
                        <!-- Theme Selection -->
                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Тема</label
                            >
                            <select
                                value={currentTheme}
                                on:change={handleThemeChange}
                                class="select-styled"
                            >
                                <option value="dark">Темная</option>
                                <option value="oled">OLED (Очень темная)</option
                                >
                            </select>
                            <p class="text-gray-400 text-xs mt-2">
                                Выберите цветовую схему приложения
                            </p>
                        </div>

                        <!-- Theme Preview -->
                        <div class="grid grid-cols-2 gap-4">
                            <div
                                class="p-4 rounded-lg border border-netflix-border"
                                style="background-color: {getThemeColors('dark')
                                    .bg}"
                            >
                                <p class="text-white text-sm font-medium">
                                    Темная
                                </p>
                                <p class="text-gray-400 text-xs mt-1">
                                    #0f0f0f
                                </p>
                            </div>
                            <div
                                class="p-4 rounded-lg border border-netflix-border"
                                style="background-color: {getThemeColors('oled')
                                    .bg}"
                            >
                                <p class="text-white text-sm font-medium">
                                    OLED
                                </p>
                                <p class="text-gray-400 text-xs mt-1">
                                    #000000
                                </p>
                            </div>
                        </div>

                        <!-- Accent Color Selection -->
                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Цвет акцента</label
                            >
                            <select
                                value={currentAccentColor}
                                on:change={handleAccentColorChange}
                                class="select-styled"
                            >
                                <option value="red">Красный</option>
                                <option value="blue">Синий</option>
                                <option value="green">Зеленый</option>
                                <option value="purple">Фиолетовый</option>
                                <option value="orange">Оранжевый</option>
                            </select>
                            <p class="text-gray-400 text-xs mt-2">
                                Выберите цвет для интерактивных элементов
                            </p>
                        </div>

                        <!-- Accent Color Preview -->
                        <div class="grid grid-cols-5 gap-2">
                            {#each ["red", "blue", "green", "purple", "orange"] as color}
                                <div
                                    class="p-3 rounded-lg border-2 cursor-pointer transition-all"
                                    style="background-color: {getAccentColorValue(
                                        color as AccentColor,
                                    )}; border-color: {currentAccentColor ===
                                    color
                                        ? '#ffffff'
                                        : 'transparent'}"
                                    on:click={() =>
                                        setAccentColor(color as AccentColor)}
                                    role="button"
                                    tabindex="0"
                                >
                                    {#if currentAccentColor === color}
                                        <p
                                            class="text-white text-xs font-bold text-center"
                                        >
                                            ✓
                                        </p>
                                    {/if}
                                </div>
                            {/each}
                        </div>
                    </div>
                </div>

                <!-- Language Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">🌐 Язык</h2>
                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-white text-sm font-medium mb-2"
                                >Язык интерфейса</label
                            >
                            <select
                                value={currentLanguage}
                                on:change={handleLanguageChange}
                                class="select-styled"
                            >
                                {#each supportedLanguages as lang}
                                    <option value={lang}>
                                        {getLanguageName(lang)}
                                    </option>
                                {/each}
                            </select>
                            <p class="text-gray-400 text-xs mt-2">
                                Выберите язык для интерфейса приложения
                            </p>
                        </div>

                        <!-- Language Info -->
                        <div
                            class="bg-netflix-dark-bg rounded-lg p-4 border border-netflix-border"
                        >
                            <p class="text-white text-sm">
                                Текущий язык: <span class="font-bold"
                                    >{getLanguageName(currentLanguage)}</span
                                >
                            </p>
                            <p class="text-gray-400 text-xs mt-2">
                                Интерфейс обновится без перезагрузки приложения
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Proxy Section -->
                <div
                    class="bg-netflix-card-bg border border-netflix-border rounded-lg p-6 mb-6"
                >
                    <h2 class="text-xl font-bold text-white mb-4">🌐 Прокси</h2>
                    <div class="space-y-4">
                        <label class="flex items-center space-x-3 mb-4">
                            <input
                                type="checkbox"
                                checked={settingsData.proxyEnabled}
                                on:change={(e) =>
                                    handleInputChange(
                                        "proxyEnabled",
                                        e.currentTarget.checked,
                                    )}
                                class="checkbox-styled"
                            />
                            <span class="text-white">Включить прокси</span>
                        </label>

                        {#if settingsData.proxyEnabled}
                            <Input
                                label="Адрес прокси"
                                type="text"
                                placeholder="127.0.0.1"
                                value={settingsData.proxyAddress || ""}
                                on:input={(e) =>
                                    handleInputEvent("proxyAddress", e)}
                                error={settingsErrors.proxyAddress}
                            />
                            <Input
                                label="Порт прокси"
                                type="number"
                                placeholder="9050"
                                value={String(settingsData.proxyPort || 9050)}
                                on:input={(e) =>
                                    handleInputEvent("proxyPort", e)}
                                error={settingsErrors.proxyPort}
                            />
                        {/if}
                    </div>
                </div>
            </div>
        </div>

        <!-- Save Button -->
        <div
            class="border-t border-netflix-border bg-netflix-card-bg px-6 py-4 flex justify-end space-x-4"
        >
            {#if dirtyState}
                <p class="text-white/70 text-sm mr-auto">
                    Есть несохраненные изменения
                </p>
            {/if}
            <Button
                variant="secondary"
                on:click={loadSettings}
                disabled={busyState}
            >
                Отменить
            </Button>
            <Button
                variant="primary"
                on:click={saveSettings}
                disabled={busyState || !dirtyState}
            >
                {busyState ? "Сохранение..." : "Сохранить"}
            </Button>
        </div>
    {/if}

    {#if showToast}
        <Toast message={toastMessage} type={toastType} />
    {/if}
</div>

<style>
    :global(.netflix-dark-bg) {
        background-color: #0f0f0f;
    }

    :global(.netflix-card-bg) {
        background-color: #1a1a1a;
    }

    :global(.netflix-border) {
        border-color: rgba(255, 255, 255, 0.1);
    }

    :global(.netflix-red) {
        color: #e50914;
    }

    :global(.checkbox-styled) {
        accent-color: #e50914;
        cursor: pointer;
    }

    :global(.select-styled) {
        width: 100%;
        background-color: #1a1a1a;
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 0.375rem;
        padding: 0.5rem 0.75rem;
        color: #ffffff;
        font-size: 1rem;
    }

    :global(.select-styled:focus) {
        outline: none;
        border-color: #e50914;
    }

    :global(.select-styled option) {
        background-color: #1a1a1a;
        color: #ffffff;
    }
</style>
