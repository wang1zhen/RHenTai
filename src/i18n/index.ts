import { create } from "zustand";

export type Language = "zh-CN" | "en";

export interface TranslationSchema {
  // Navigation & Layout
  home: string;
  popular: string;
  toplist: string;
  watched: string;
  favorites: string;
  downloads: string;
  library: string;
  history: string;
  settings: string;
  login: string;
  logout: string;
  search: string;
  searchPlaceholder: string;
  site: string;
  theme: string;
  language: string;
  account: string;
  loggedInAs: string;
  notLoggedIn: string;
  exAccess: string;
  noExAccess: string;

  // Categories
  doujinshi: string;
  manga: string;
  artistCg: string;
  gameCg: string;
  western: string;
  nonH: string;
  imageSet: string;
  cosplay: string;
  asianPorn: string;
  misc: string;
  private: string;
  unknown: string;

  // Periods
  day: string;
  month: string;
  year: string;
  all: string;

  // Gallery Card & Detail
  pages: string;
  galleriesLoaded: string;
  uploader: string;
  posted: string;
  read: string;
  continueReading: string;
  download: string;
  favorite: string;
  favorited: string;
  openInBrowser: string;
  comments: string;
  noComments: string;
  rating: string;
  fileSize: string;
  favoritedTimes: string;
  tags: string;
  noTags: string;
  parentGallery: string;
  thumbnailPage: string;
  prevPage: string;
  nextPage: string;
  loadMore: string;
  loadingMore: string;
  noMoreItems: string;
  details: string;
  score: string;

  // Search & Filter
  advancedFilter: string;
  filterCategories: string;
  minRating: string;
  minPages: string;
  maxPages: string;
  showExpunged: string;
  requireTorrent: string;
  apply: string;
  reset: string;
  selectAll: string;
  clearAll: string;
  noResults: string;
  noResultsDesc: string;
  anyRating: string;
  starsOrMore: string;

  // Multi-select & Batch
  selectMode: string;
  selectedCount: string;
  cancelSelect: string;
  batchDownload: string;
  batchMoveFavorite: string;
  batchRemoveFavorite: string;
  confirmBatchDelete: string;

  // Favorites
  allFavorites: string;
  favoriteSlot: string;
  favoriteNote: string;
  favoriteNotePlaceholder: string;
  saveFavorite: string;
  removeFromFavorites: string;
  loginRequiredFavorites: string;
  loginRequiredFavoritesDesc: string;
  loginRequiredWatched: string;
  loginRequiredWatchedDesc: string;
  goToLogin: string;

  // Downloads
  downloadTasks: string;
  pause: string;
  resume: string;
  remove: string;
  pauseAll: string;
  resumeAll: string;
  clearCompleted: string;
  speed: string;
  queued: string;
  running: string;
  paused: string;
  completed: string;
  failed: string;
  noDownloads: string;
  noDownloadsDesc: string;
  openFolder: string;
  deleteFiles: string;
  deleteFilesDesc: string;
  confirmRemoveDownload: string;

  // Library
  localGalleries: string;
  filterLibrary: string;
  rescan: string;
  rescanning: string;
  addFolder: string;
  removeGallery: string;
  confirmRemoveLibrary: string;
  folder: string;
  archive: string;
  noLocalGalleries: string;
  noLocalGalleriesDesc: string;

  // History
  readingHistory: string;
  clearHistory: string;
  confirmClearHistory: string;
  noHistory: string;
  noHistoryDesc: string;
  readProgress: string;

  // Reader
  readerMode: string;
  vertical: string;
  paged: string;
  direction: string;
  ltr: string;
  rtl: string;
  fit: string;
  fitWidth: string;
  fitHeight: string;
  fitOriginal: string;
  preload: string;
  pageError: string;
  retry: string;
  back: string;
  pageIndicator: string;
  pageJump: string;
  readingSettings: string;

  // Settings
  siteSettings: string;
  proxySettings: string;
  proxyPlaceholder: string;
  downloadSettings: string;
  downloadDir: string;
  browse: string;
  concurrency: string;
  downloadOriginal: string;
  readerSettings: string;
  appearanceSettings: string;
  systemTheme: string;
  lightTheme: string;
  darkTheme: string;
  libraryFolders: string;
  addPath: string;
  removePath: string;
  cacheSettings: string;
  clearImageCache: string;
  cacheClearedToast: string;
  saveSuccess: string;
  saveFailed: string;
  accountSettings: string;
  save: string;

  // Login
  webLogin: string;
  cookieLogin: string;
  passwordLogin: string;
  openLoginWindow: string;
  loginWindowHint: string;
  username: string;
  password: string;
  memberId: string;
  passHash: string;
  igneous: string;
  igneousOptional: string;
  loggingIn: string;
  cloudflareHintTitle: string;
  cloudflareHintDesc: string;
  loginSuccess: string;

  // Error States
  errorTitle: string;
  errorNetwork: string;
  errorHttp: string;
  errorParse: string;
  errorCloudflare: string;
  errorBanned: string;
  errorLoginRequired: string;
  errorLoginFailed: string;
  errorNoExAccess: string;
  errorImageLimit: string;
  errorNotFound: string;
  errorInvalidInput: string;
  errorIo: string;
  errorDb: string;
  errorCancelled: string;
  errorInternal: string;
  unknownError: string;
  retryAction: string;

  // Common UI
  cancel: string;
  confirm: string;
  close: string;
  loading: string;
  success: string;
  warning: string;
}

export const translations: Record<Language, TranslationSchema> = {
  "zh-CN": {
    home: "首页",
    popular: "热门",
    toplist: "排行",
    watched: "关注",
    favorites: "收藏",
    downloads: "下载",
    library: "书库",
    history: "历史",
    settings: "设置",
    login: "登录",
    logout: "登出",
    search: "搜索",
    searchPlaceholder: "输入关键字或标签...",
    site: "站点",
    theme: "主题",
    language: "语言",
    account: "账号",
    loggedInAs: "已登录为",
    notLoggedIn: "未登录",
    exAccess: "ExHentai 访问权限有效",
    noExAccess: "无 ExHentai 权限",

    doujinshi: "同人志",
    manga: "漫画",
    artistCg: "画师CG",
    gameCg: "游戏CG",
    western: "欧美",
    nonH: "非H",
    imageSet: "图集",
    cosplay: "Cosplay",
    asianPorn: "亚洲写真",
    misc: "杂项",
    private: "私有",
    unknown: "未知",

    day: "日榜",
    month: "月榜",
    year: "年榜",
    all: "总榜",

    pages: "页",
    galleriesLoaded: "已加载 {n} 个画廊",
    uploader: "上传者",
    posted: "发布时间",
    read: "阅读",
    continueReading: "继续阅读",
    download: "下载",
    favorite: "收藏",
    favorited: "已收藏",
    openInBrowser: "浏览器中打开",
    comments: "评论",
    noComments: "暂无评论",
    rating: "评分",
    fileSize: "大小",
    favoritedTimes: "次收藏",
    tags: "标签",
    noTags: "暂无标签",
    parentGallery: "父画廊",
    thumbnailPage: "缩略图",
    prevPage: "上一页",
    nextPage: "下一页",
    loadMore: "加载更多",
    loadingMore: "加载中...",
    noMoreItems: "没有更多画廊了",
    details: "画廊详情",
    score: "评分",

    advancedFilter: "高级筛选",
    filterCategories: "排除分类",
    minRating: "最低评分",
    minPages: "最小页数",
    maxPages: "最大页数",
    showExpunged: "显示已废弃",
    requireTorrent: "需要种子",
    apply: "应用",
    reset: "重置",
    selectAll: "全选",
    clearAll: "清除",
    noResults: "未找到画廊",
    noResultsDesc: "尝试调整筛选条件或搜索关键词",
    anyRating: "不限",
    starsOrMore: "星以上",

    selectMode: "批量选择",
    selectedCount: "已选择 {count} 项",
    cancelSelect: "取消选择",
    batchDownload: "批量下载",
    batchMoveFavorite: "移至收藏夹",
    batchRemoveFavorite: "移出收藏夹",
    confirmBatchDelete: "确认将所选项移出收藏夹？",

    allFavorites: "全部收藏",
    favoriteSlot: "收藏夹分组",
    favoriteNote: "备注",
    favoriteNotePlaceholder: "添加备注文档（可选）...",
    saveFavorite: "保存收藏",
    removeFromFavorites: "移出收藏夹",
    loginRequiredFavorites: "需要登录以查看收藏夹",
    loginRequiredFavoritesDesc: "登录你的 E-Hentai / ExHentai 账号以同步云端收藏夹与分组。",
    loginRequiredWatched: "需要登录以查看关注标签",
    loginRequiredWatchedDesc: "登录后即可查看你在站点上设置的关注更新。",
    goToLogin: "前往登录",

    downloadTasks: "下载管理",
    pause: "暂停",
    resume: "继续",
    remove: "删除",
    pauseAll: "全部暂停",
    resumeAll: "全部开始",
    clearCompleted: "清除已完成",
    speed: "速度",
    queued: "排队中",
    running: "下载中",
    paused: "已暂停",
    completed: "已完成",
    failed: "已失败",
    noDownloads: "暂无下载任务",
    noDownloadsDesc: "浏览画廊时点击下载，即可在此查看进度并离线阅读。",
    openFolder: "打开目录",
    deleteFiles: "同时删除本地文件",
    deleteFilesDesc: "此操作将从磁盘中彻底清除画廊图片文件。",
    confirmRemoveDownload: "确认删除此下载任务？",

    localGalleries: "本地书库",
    filterLibrary: "筛选本地书库...",
    rescan: "重新扫描",
    rescanning: "扫描中...",
    addFolder: "添加目录",
    removeGallery: "移除书库条目",
    confirmRemoveLibrary: "确认从书库中移除此条目？",
    folder: "文件夹",
    archive: "压缩包",
    noLocalGalleries: "本地书库为空",
    noLocalGalleriesDesc: "添加本地漫画目录或下载画廊后，将在此展示。",

    readingHistory: "阅读历史",
    clearHistory: "清空历史",
    confirmClearHistory: "确认清空所有阅读历史记录？",
    noHistory: "暂无历史记录",
    noHistoryDesc: "你阅读过的画廊会自动保存在这里，方便下次继续阅读。",
    readProgress: "阅读至第 {page} / {total} 页",

    readerMode: "阅读模式",
    vertical: "纵向连页",
    paged: "单页翻页",
    direction: "翻页方向",
    ltr: "从左到右 (LTR)",
    rtl: "从右到左 (RTL)",
    fit: "画面适配",
    fitWidth: "适应宽度",
    fitHeight: "适应高度",
    fitOriginal: "原始尺寸",
    preload: "预加载页数",
    pageError: "图片加载失败",
    retry: "重试加载",
    back: "返回",
    pageIndicator: "第 {current} / {total} 页",
    pageJump: "跳转至页码",
    readingSettings: "阅读设置",

    siteSettings: "站点设置",
    proxySettings: "网络代理",
    proxyPlaceholder: "例如 http://127.0.0.1:7890 或 socks5://127.0.0.1:1080",
    downloadSettings: "下载设置",
    downloadDir: "下载存储路径",
    browse: "浏览",
    concurrency: "最大并发任务数",
    downloadOriginal: "优先下载原图",
    readerSettings: "默认阅读偏好",
    appearanceSettings: "外观偏好",
    systemTheme: "跟随系统",
    lightTheme: "浅色模式",
    darkTheme: "深色模式",
    libraryFolders: "本地书库包含目录",
    addPath: "添加路径",
    removePath: "移除路径",
    cacheSettings: "缓存管理",
    clearImageCache: "清除图片缓存",
    cacheClearedToast: "已成功清除缓存：{bytes}",
    saveSuccess: "设置已保存",
    saveFailed: "保存设置失败",
    accountSettings: "账号管理",
    save: "保存更改",

    webLogin: "网页登录 (推荐)",
    cookieLogin: "Cookie 登录",
    passwordLogin: "账号密码登录",
    openLoginWindow: "打开登录窗口",
    loginWindowHint: "将在桌面弹出 E-Hentai 论坛登录页面。登录完成后窗口将自动关闭并导入凭证。",
    username: "用户名",
    password: "密码",
    memberId: "用户 ID (ipb_member_id)",
    passHash: "密码散列 (ipb_pass_hash)",
    igneous: "Igneous (ExHentai 必要 Cookie)",
    igneousOptional: "可选，留空则尝试自动获取",
    loggingIn: "正在登录...",
    cloudflareHintTitle: "检测到 Cloudflare 人机验证",
    cloudflareHintDesc: "站点当前受到 Cloudflare 防护拦截，请使用「网页登录」或手动填写 Cookie 登录。",
    loginSuccess: "登录成功",

    errorTitle: "操作失败",
    errorNetwork: "网络连接失败，请检查网络或代理设置",
    errorHttp: "服务器返回 HTTP 异常状态码",
    errorParse: "解析页面内容失败，可能是站点结构变更",
    errorCloudflare: "遭遇 Cloudflare 防护拦截",
    errorBanned: "当前 IP 已被站点封禁",
    errorLoginRequired: "此操作需要登录账号",
    errorLoginFailed: "登录验证失败，请核对凭证",
    errorNoExAccess: "该账号没有 ExHentai 里站访问权限",
    errorImageLimit: "配额超限：已达到当前图片配额限制 (509)",
    errorNotFound: "未找到目标内容",
    errorInvalidInput: "输入参数无效",
    errorIo: "文件或磁盘输入输出错误",
    errorDb: "本地数据库操作错误",
    errorCancelled: "任务已取消",
    errorInternal: "程序内部发生异常",
    unknownError: "发生未知错误",
    retryAction: "重试",

    cancel: "取消",
    confirm: "确认",
    close: "关闭",
    loading: "加载中...",
    success: "成功",
    warning: "提示",
  },
  en: {
    home: "Home",
    popular: "Popular",
    toplist: "Toplist",
    watched: "Watched",
    favorites: "Favorites",
    downloads: "Downloads",
    library: "Library",
    history: "History",
    settings: "Settings",
    login: "Login",
    logout: "Logout",
    search: "Search",
    searchPlaceholder: "Search keywords or tags...",
    site: "Site",
    theme: "Theme",
    language: "Language",
    account: "Account",
    loggedInAs: "Logged in as",
    notLoggedIn: "Not logged in",
    exAccess: "ExHentai access active",
    noExAccess: "No ExHentai access",

    doujinshi: "Doujinshi",
    manga: "Manga",
    artistCg: "Artist CG",
    gameCg: "Game CG",
    western: "Western",
    nonH: "Non-H",
    imageSet: "Image Set",
    cosplay: "Cosplay",
    asianPorn: "Asian Porn",
    misc: "Misc",
    private: "Private",
    unknown: "Unknown",

    day: "Day",
    month: "Month",
    year: "Year",
    all: "All",

    pages: "pages",
    galleriesLoaded: "{n} galleries loaded",
    uploader: "Uploader",
    posted: "Posted",
    read: "Read",
    continueReading: "Continue",
    download: "Download",
    favorite: "Favorite",
    favorited: "Favorited",
    openInBrowser: "Open in Browser",
    comments: "Comments",
    noComments: "No comments yet",
    rating: "Rating",
    fileSize: "Size",
    favoritedTimes: "favorites",
    tags: "Tags",
    noTags: "No tags",
    parentGallery: "Parent Gallery",
    thumbnailPage: "Thumbnails",
    prevPage: "Previous",
    nextPage: "Next",
    loadMore: "Load More",
    loadingMore: "Loading...",
    noMoreItems: "No more galleries",
    details: "Gallery Details",
    score: "Score",

    advancedFilter: "Advanced Filters",
    filterCategories: "Excluded Categories",
    minRating: "Minimum Rating",
    minPages: "Min Pages",
    maxPages: "Max Pages",
    showExpunged: "Show Expunged",
    requireTorrent: "Require Torrents",
    apply: "Apply",
    reset: "Reset",
    selectAll: "Select All",
    clearAll: "Clear",
    noResults: "No galleries found",
    noResultsDesc: "Try adjusting your filters or search terms",
    anyRating: "Any",
    starsOrMore: "stars or more",

    selectMode: "Batch Select",
    selectedCount: "{count} selected",
    cancelSelect: "Cancel",
    batchDownload: "Download Selected",
    batchMoveFavorite: "Move to Favorite Slot",
    batchRemoveFavorite: "Remove from Favorites",
    confirmBatchDelete: "Remove selected galleries from favorites?",

    allFavorites: "All Favorites",
    favoriteSlot: "Favorite Slot",
    favoriteNote: "Note",
    favoriteNotePlaceholder: "Add an optional note...",
    saveFavorite: "Save to Favorites",
    removeFromFavorites: "Remove Favorite",
    loginRequiredFavorites: "Login Required for Favorites",
    loginRequiredFavoritesDesc: "Log in with your E-Hentai / ExHentai account to sync cloud favorites and categories.",
    loginRequiredWatched: "Login Required for Watched",
    loginRequiredWatchedDesc: "Log in to browse the tags and galleries you follow on E-Hentai.",
    goToLogin: "Go to Login",

    downloadTasks: "Downloads",
    pause: "Pause",
    resume: "Resume",
    remove: "Remove",
    pauseAll: "Pause All",
    resumeAll: "Resume All",
    clearCompleted: "Clear Completed",
    speed: "Speed",
    queued: "Queued",
    running: "Downloading",
    paused: "Paused",
    completed: "Completed",
    failed: "Failed",
    noDownloads: "No downloads",
    noDownloadsDesc: "Galleries you download will appear here for offline reading.",
    openFolder: "Open Folder",
    deleteFiles: "Delete files on disk",
    deleteFilesDesc: "This will permanently remove image files from your computer.",
    confirmRemoveDownload: "Remove this download task?",

    localGalleries: "Local Library",
    filterLibrary: "Filter library...",
    rescan: "Rescan",
    rescanning: "Scanning...",
    addFolder: "Add Folder",
    removeGallery: "Remove Entry",
    confirmRemoveLibrary: "Remove this gallery from library?",
    folder: "Folder",
    archive: "Archive",
    noLocalGalleries: "Library is empty",
    noLocalGalleriesDesc: "Add local folders or download galleries to view them here.",

    readingHistory: "Reading History",
    clearHistory: "Clear History",
    confirmClearHistory: "Are you sure you want to clear your reading history?",
    noHistory: "No history yet",
    noHistoryDesc: "Galleries you open will be remembered here so you can pick up where you left off.",
    readProgress: "Read to page {page} of {total}",

    readerMode: "Reader Mode",
    vertical: "Vertical Continuous",
    paged: "Paged",
    direction: "Page Direction",
    ltr: "Left to Right (LTR)",
    rtl: "Right to Left (RTL)",
    fit: "Image Fit",
    fitWidth: "Fit Width",
    fitHeight: "Fit Height",
    fitOriginal: "Original Size",
    preload: "Preload Pages",
    pageError: "Failed to load page image",
    retry: "Retry",
    back: "Back",
    pageIndicator: "Page {current} / {total}",
    pageJump: "Jump to page",
    readingSettings: "Reader Settings",

    siteSettings: "Site Settings",
    proxySettings: "Proxy Settings",
    proxyPlaceholder: "e.g. http://127.0.0.1:7890 or socks5://127.0.0.1:1080",
    downloadSettings: "Download Settings",
    downloadDir: "Download Directory",
    browse: "Browse",
    concurrency: "Download Concurrency",
    downloadOriginal: "Download Original Images",
    readerSettings: "Default Reader Preferences",
    appearanceSettings: "Appearance",
    systemTheme: "System Default",
    lightTheme: "Light Mode",
    darkTheme: "Dark Mode",
    libraryFolders: "Library Directories",
    addPath: "Add Directory",
    removePath: "Remove Directory",
    cacheSettings: "Cache Management",
    clearImageCache: "Clear Image Cache",
    cacheClearedToast: "Successfully cleared cache: {bytes}",
    saveSuccess: "Settings saved",
    saveFailed: "Failed to save settings",
    accountSettings: "Account",
    save: "Save Changes",

    webLogin: "Web Login (Recommended)",
    cookieLogin: "Cookie Login",
    passwordLogin: "Password Login",
    openLoginWindow: "Open Login Window",
    loginWindowHint: "A desktop window will open the E-Hentai forums login page. Once logged in, credentials will be automatically captured.",
    username: "Username",
    password: "Password",
    memberId: "Member ID (ipb_member_id)",
    passHash: "Password Hash (ipb_pass_hash)",
    igneous: "Igneous Cookie (ExHentai)",
    igneousOptional: "Optional; leave blank to attempt automatic retrieval",
    loggingIn: "Logging in...",
    cloudflareHintTitle: "Cloudflare Challenge Detected",
    cloudflareHintDesc: "The site is currently protected by Cloudflare. Please use Web Login or paste your cookies directly.",
    loginSuccess: "Login successful",

    errorTitle: "Operation Failed",
    errorNetwork: "Network connection failed. Please check your network or proxy settings.",
    errorHttp: "Server returned an HTTP error status.",
    errorParse: "Failed to parse page content. The site layout may have changed.",
    errorCloudflare: "Blocked by Cloudflare security check.",
    errorBanned: "Your IP address is currently banned by the site.",
    errorLoginRequired: "This action requires an active login session.",
    errorLoginFailed: "Authentication failed. Please verify your credentials.",
    errorNoExAccess: "This account does not have ExHentai access privileges.",
    errorImageLimit: "Image quota exceeded (509 Image Limit).",
    errorNotFound: "Requested item not found.",
    errorInvalidInput: "Invalid input provided.",
    errorIo: "Disk or file input/output error.",
    errorDb: "Database operation failed.",
    errorCancelled: "Operation was cancelled.",
    errorInternal: "An internal application error occurred.",
    unknownError: "An unexpected error occurred.",
    retryAction: "Retry",

    cancel: "Cancel",
    confirm: "Confirm",
    close: "Close",
    loading: "Loading...",
    success: "Success",
    warning: "Warning",
  },
};

export type TranslationKey = keyof TranslationSchema;

interface I18nStore {
  language: Language;
  setLanguage: (lang: Language) => void;
}

export const useI18nStore = create<I18nStore>((set) => ({
  language: "zh-CN",
  setLanguage: (language) => set({ language }),
}));

export function useT() {
  const { language, setLanguage } = useI18nStore();
  const t = (key: TranslationKey, params?: Record<string, string | number>): string => {
    let text: string = translations[language][key] || translations["zh-CN"][key] || key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(new RegExp(`\\{${k}\\}`, "g"), String(v));
      }
    }
    return text;
  };
  return { t, language, setLanguage };
}
