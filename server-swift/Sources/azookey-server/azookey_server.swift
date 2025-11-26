import KanaKanjiConverterModule
import Foundation
#if canImport(FoundationNetworking)
import FoundationNetworking
#endif
import ffi

// スレッドセーフなIME状態管理
// @MainActorの代わりにDispatchQueueを使用してスレッドセーフにする
final class IMEState: @unchecked Sendable {
    static let shared = IMEState()
    
    private let queue = DispatchQueue(label: "com.azookey.ime", qos: .userInteractive)
    
    private var _converter: KanaKanjiConverter
    private var _composingText: ComposingText
    private var _execURL: URL
    private var _config: [String: Any]
    private var _iikanjiResult: String?
    private var _iikanjiError: String?
    private var _memoryURL: URL
    private var _userDictionaryURL: URL
    
    private init() {
        _converter = KanaKanjiConverter()
        _composingText = ComposingText()
        _execURL = URL(filePath: "")
        _config = [
            "enable": false,
            "profile": "",
            "learningEnabled": true,
            "predictionEnabled": true,
            "shouldResetMemory": false,
            "iikanjiEnabled": false,
            "iikanjiProvider": "zenzai",
            "openaiApiKey": "",
            "openaiModel": "gpt-4o-mini",
            "openaiMaxTokens": 256,
            "openaiTemperature": 0.7,
        ]
        _iikanjiResult = nil
        _iikanjiError = nil
        
        // 学習データの保存先ディレクトリ
        if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
            _memoryURL = URL(filePath: appDataPath).appendingPathComponent("Azookey/memory")
            try? FileManager.default.createDirectory(at: _memoryURL, withIntermediateDirectories: true)
        } else {
            _memoryURL = URL(filePath: "./memory")
        }
        
        // ユーザー辞書の保存先ディレクトリ
        if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
            _userDictionaryURL = URL(filePath: appDataPath).appendingPathComponent("Azookey/user_dictionary")
            try? FileManager.default.createDirectory(at: _userDictionaryURL, withIntermediateDirectories: true)
        } else {
            _userDictionaryURL = URL(filePath: "./user_dictionary")
        }
    }
    
    // スレッドセーフにブロックを実行
    func sync<T>(_ block: () throws -> T) rethrows -> T {
        return try queue.sync { try block() }
    }
    
    // アクセサ
    var converter: KanaKanjiConverter {
        get { queue.sync { _converter } }
    }
    
    var composingText: ComposingText {
        get { queue.sync { _composingText } }
        set { queue.sync { _composingText = newValue } }
    }
    
    func modifyComposingText(_ block: (inout ComposingText) -> Void) {
        queue.sync { block(&_composingText) }
    }
    
    var execURL: URL {
        get { queue.sync { _execURL } }
        set { queue.sync { _execURL = newValue } }
    }
    
    var config: [String: Any] {
        get { queue.sync { _config } }
        set { queue.sync { _config = newValue } }
    }
    
    func getConfigValue<T>(_ key: String, default defaultValue: T) -> T {
        return queue.sync { (_config[key] as? T) ?? defaultValue }
    }
    
    func setConfigValue(_ key: String, _ value: Any) {
        queue.sync { _config[key] = value }
    }
    
    var iikanjiResult: String? {
        get { queue.sync { _iikanjiResult } }
        set { queue.sync { _iikanjiResult = newValue } }
    }
    
    var iikanjiError: String? {
        get { queue.sync { _iikanjiError } }
        set { queue.sync { _iikanjiError = newValue } }
    }
    
    var memoryURL: URL {
        get { queue.sync { _memoryURL } }
    }
    
    var userDictionaryURL: URL {
        get { queue.sync { _userDictionaryURL } }
    }
    
    // converter.requestCandidatesをスレッドセーフに実行
    func requestCandidates(_ composingText: ComposingText, options: ConvertRequestOptions) -> ConvertRequestOptions.ReturningResultType {
        return queue.sync {
            return _converter.requestCandidates(composingText, options: options)
        }
    }
    
    // IME状態を使った操作をスレッドセーフに実行
    func getOptions(context: String = "") -> ConvertRequestOptions {
        return queue.sync {
            let learningEnabled = (_config["learningEnabled"] as? Bool) ?? true
            let predictionEnabled = (_config["predictionEnabled"] as? Bool) ?? true
            let shouldReset = (_config["shouldResetMemory"] as? Bool) ?? false
            
            if shouldReset {
                _config["shouldResetMemory"] = false
            }
            
            let zenzaiEnabled = (_config["enable"] as? Bool) ?? false
            let profile = (_config["profile"] as? String) ?? ""
            
            return ConvertRequestOptions(
                requireJapanesePrediction: predictionEnabled,
                requireEnglishPrediction: false,
                keyboardLanguage: .ja_JP,
                learningType: learningEnabled ? .inputAndOutput : .nothing,
                maxMemoryCount: 65536,
                shouldResetMemory: shouldReset,
                dictionaryResourceURL: _execURL.appendingPathComponent("Dictionary"),
                memoryDirectoryURL: _memoryURL,
                sharedContainerURL: _userDictionaryURL,
                textReplacer: .init {
                    return self._execURL.appendingPathComponent("EmojiDictionary").appendingPathComponent("emoji_all_E15.1.txt")
                },
                zenzaiMode: zenzaiEnabled ? .on(
                    weight: _execURL.appendingPathComponent("zenz.gguf"),
                    inferenceLimit: 1,
                    requestRichCandidates: true,
                    personalizationMode: nil,
                    versionDependentMode: .v3(
                        .init(
                            profile: profile,
                            leftSideContext: context
                        )
                    )
                ) : .off,
                preloadDictionary: true,
                metadata: .init(versionString: "Azookey for Windows")
            )
        }
    }
}

// いい感じ変換キーワード定義
enum IikanjiKeyword: String, CaseIterable {
    case eigo = "えいご"
    case nihongo = "にほんご"
    case emoji = "えもじ"
    case iikae = "いいかえ"
    case keigo = "けいご"
    case tamego = "ためご"
    case kousei = "こうせい"
    
    var katakanaVariant: String {
        switch self {
        case .eigo: return "エイゴ"
        case .nihongo: return "ニホンゴ"
        case .emoji: return "エモジ"
        case .iikae: return "イイカエ"
        case .keigo: return "ケイゴ"
        case .tamego: return "タメゴ"
        case .kousei: return "コウセイ"
        }
    }
    
    var prompt: String {
        switch self {
        case .eigo:
            return "Translate the following Japanese text to natural English. Output only the translation, nothing else:"
        case .nihongo:
            return "以下の英語を自然な日本語に翻訳してください。翻訳のみ出力してください:"
        case .emoji:
            return "以下の文脈に最も適した絵文字を1-3個提案してください。絵文字のみ出力してください:"
        case .iikae:
            return "以下の文を別の表現で言い換えてください。言い換えのみ出力してください:"
        case .keigo:
            return "以下の文を丁寧な敬語に変換してください。変換結果のみ出力してください:"
        case .tamego:
            return "以下の文をカジュアルなため口に変換してください。変換結果のみ出力してください:"
        case .kousei:
            return "以下の文の文法・誤字脱字を校正してください。校正結果のみ出力してください:"
        }
    }
    
    var zenzaiPrompt: String {
        switch self {
        case .eigo:
            return "を英語に翻訳:"
        case .nihongo:
            return "を日本語に翻訳:"
        case .emoji:
            return "に合う絵文字:"
        case .iikae:
            return "を言い換え:"
        case .keigo:
            return "を敬語に:"
        case .tamego:
            return "をカジュアルに:"
        case .kousei:
            return "を校正:"
        }
    }
    
    static func detect(_ input: String) -> IikanjiKeyword? {
        let normalized = input.trimmingCharacters(in: .whitespaces)
        for keyword in IikanjiKeyword.allCases {
            if normalized == keyword.rawValue || normalized == keyword.katakanaVariant {
                return keyword
            }
        }
        return nil
    }
}

// ヘルパー関数
func constructCandidateString(candidate: Candidate, hiragana: String) -> String {
    var remainingHiragana = hiragana
    var result = ""
    
    for data in candidate.data {
        if remainingHiragana.count < data.ruby.count {
            result += remainingHiragana
            break
        }
        remainingHiragana.removeFirst(data.ruby.count)
        result += data.word
    }
    
    return result
}

func to_list_pointer(_ list: [FFICandidate]) -> UnsafeMutablePointer<UnsafeMutablePointer<FFICandidate>?> {
    let pointer = UnsafeMutablePointer<UnsafeMutablePointer<FFICandidate>?>.allocate(capacity: max(list.count, 1))
    for (i, item) in list.enumerated() {
        pointer[i] = UnsafeMutablePointer<FFICandidate>.allocate(capacity: 1)
        pointer[i]?.pointee = item
    }
    return pointer
}

// FFI関数 - @MainActorを削除してスレッドセーフに

@_silgen_name("LoadConfig")
public func load_config() {
    let state = IMEState.shared
    
    if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
        let settingsPath = URL(filePath: appDataPath).appendingPathComponent("Azookey/settings.json")
        
        do {
            let data = try Data(contentsOf: settingsPath)
            if let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] {
                if let zenzaiDict = json["zenzai"] as? [String: Any] {
                    if let enableValue = zenzaiDict["enable"] as? Bool {
                        state.setConfigValue("enable", enableValue)
                    }
                    if let profileValue = zenzaiDict["profile"] as? String {
                        state.setConfigValue("profile", profileValue)
                    }
                }
                
                if let learningDict = json["learning"] as? [String: Any] {
                    if let enabledValue = learningDict["enabled"] as? Bool {
                        state.setConfigValue("learningEnabled", enabledValue)
                    }
                }
                
                if let predictionDict = json["prediction"] as? [String: Any] {
                    if let enabledValue = predictionDict["enabled"] as? Bool {
                        state.setConfigValue("predictionEnabled", enabledValue)
                    }
                }
                
                if let iikanjiDict = json["iikanji"] as? [String: Any] {
                    if let enabledValue = iikanjiDict["enabled"] as? Bool {
                        state.setConfigValue("iikanjiEnabled", enabledValue)
                    }
                    if let providerValue = iikanjiDict["provider"] as? String {
                        state.setConfigValue("iikanjiProvider", providerValue)
                    }
                    if let openaiDict = iikanjiDict["openai"] as? [String: Any] {
                        if let apiKeyValue = openaiDict["api_key"] as? String {
                            state.setConfigValue("openaiApiKey", apiKeyValue)
                        }
                        if let modelValue = openaiDict["model"] as? String {
                            state.setConfigValue("openaiModel", modelValue)
                        }
                        if let maxTokensValue = openaiDict["max_tokens"] as? Int {
                            state.setConfigValue("openaiMaxTokens", maxTokensValue)
                        }
                        if let temperatureValue = openaiDict["temperature"] as? Double {
                            state.setConfigValue("openaiTemperature", temperatureValue)
                        }
                    }
                }
            }
        } catch {
            print("Failed to read settings: \(error)")
        }
    }
}

@_silgen_name("Initialize")
public func initialize(
    path: UnsafePointer<CChar>,
    use_zenzai: Bool
) {
    let state = IMEState.shared
    let pathString = String(cString: path)
    state.execURL = URL(filePath: pathString)
    
    load_config()
    
    // 初期化のための変換実行
    state.sync {
        var tempComposingText = ComposingText()
        tempComposingText.insertAtCursorPosition("a", inputStyle: .roman2kana)
        let options = state.getOptions()
        _ = state.requestCandidates(tempComposingText, options: options)
    }
    state.composingText = ComposingText()
}

@_silgen_name("AppendText")
public func append_text(
    input: UnsafePointer<CChar>,
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let state = IMEState.shared
    let inputString = String(cString: input)
    
    return state.sync {
        var composingText = state.composingText
        composingText.insertAtCursorPosition(inputString, inputStyle: .roman2kana)
        state.composingText = composingText
        
        cursorPtr.pointee = composingText.convertTargetCursorPosition
        return _strdup(composingText.convertTarget)!
    }
}

@_silgen_name("RemoveText")
public func remove_text(
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let state = IMEState.shared
    
    return state.sync {
        var composingText = state.composingText
        composingText.deleteBackwardFromCursorPosition(count: 1)
        state.composingText = composingText
        
        cursorPtr.pointee = composingText.convertTargetCursorPosition
        return _strdup(composingText.convertTarget)!
    }
}

@_silgen_name("MoveCursor")
public func move_cursor(
    offset: Int32,
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let state = IMEState.shared
    
    return state.sync {
        var composingText = state.composingText
        let cursor = composingText.moveCursorFromCursorPosition(count: Int(offset))
        state.composingText = composingText
        print("offset: \(offset), cursor: \(cursor)")
        
        cursorPtr.pointee = cursor
        return _strdup(composingText.convertTarget)!
    }
}

@_silgen_name("ClearText")
public func clear_text() {
    IMEState.shared.composingText = ComposingText()
}

@_silgen_name("GetComposedText")
public func get_composed_text(lengthPtr: UnsafeMutablePointer<Int>) -> UnsafeMutablePointer<UnsafeMutablePointer<FFICandidate>?> {
    let state = IMEState.shared
    
    return state.sync {
        let composingText = state.composingText
        let hiragana = composingText.convertTarget
        let contextString = state.getConfigValue("context", default: "")
        let options = state.getOptions(context: contextString)
        let converted = state.requestCandidates(composingText, options: options)
        
        var result: [FFICandidate] = []
        
        for candidate in converted.mainResults {
            let text = strdup(constructCandidateString(candidate: candidate, hiragana: hiragana))
            let hiraganaPtr = strdup(hiragana)
            let correspondingCount = candidate.correspondingCount
            
            var afterComposingText = composingText
            afterComposingText.prefixComplete(correspondingCount: correspondingCount)
            let subtext = strdup(afterComposingText.convertTarget)
            
            result.append(FFICandidate(text: text, subtext: subtext, hiragana: hiraganaPtr, correspondingCount: Int32(correspondingCount)))
        }
        
        lengthPtr.pointee = result.count
        return to_list_pointer(result)
    }
}

@_silgen_name("ShrinkText")
public func shrink_text(
    offset: Int32
) -> UnsafeMutablePointer<CChar> {
    let state = IMEState.shared
    
    return state.sync {
        var composingText = state.composingText
        composingText.prefixComplete(correspondingCount: Int(offset))
        state.composingText = composingText
        
        return _strdup(composingText.convertTarget)!
    }
}

@_silgen_name("SetContext")
public func set_context(
    context: UnsafePointer<CChar>
) {
    let contextString = String(cString: context)
    IMEState.shared.setConfigValue("context", contextString)
}

@_silgen_name("ResetLearning")
public func reset_learning() {
    let state = IMEState.shared
    state.setConfigValue("shouldResetMemory", true)
    
    // 即座にリセットを反映するためにダミーの変換を実行
    state.sync {
        var tempComposingText = ComposingText()
        tempComposingText.insertAtCursorPosition("a", inputStyle: .roman2kana)
        _ = state.requestCandidates(tempComposingText, options: state.getOptions())
    }
    
    print("Learning data reset requested")
}

// いい感じ変換FFI関数

@_silgen_name("IsIikanjiKeyword")
public func is_iikanji_keyword(
    inputPtr: UnsafePointer<CChar>
) -> Bool {
    let state = IMEState.shared
    let enabled = state.getConfigValue("iikanjiEnabled", default: false)
    guard enabled else {
        return false
    }
    
    let input = String(cString: inputPtr)
    return IikanjiKeyword.detect(input) != nil
}

@_silgen_name("IsIikanjiEnabled")
public func is_iikanji_enabled() -> Bool {
    return IMEState.shared.getConfigValue("iikanjiEnabled", default: false)
}

@_silgen_name("RequestIikanji")
public func request_iikanji(
    keywordPtr: UnsafePointer<CChar>,
    contextPtr: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>? {
    let state = IMEState.shared
    let keywordStr = String(cString: keywordPtr)
    let contextStr = String(cString: contextPtr)
    
    guard let keyword = IikanjiKeyword.detect(keywordStr) else {
        print("Unknown iikanji keyword: \(keywordStr)")
        return nil
    }
    
    let enabled = state.getConfigValue("iikanjiEnabled", default: false)
    guard enabled else {
        print("Iikanji is disabled")
        return nil
    }
    
    guard !contextStr.isEmpty else {
        print("Context is empty")
        state.iikanjiError = "変換対象のテキストがありません"
        return nil
    }
    
    let provider = state.getConfigValue("iikanjiProvider", default: "zenzai")
    
    var result: String? = nil
    if provider == "zenzai" {
        result = requestIikanjiWithZenzai(keyword: keyword, context: contextStr)
    } else {
        result = requestIikanjiWithOpenAI(keyword: keyword, context: contextStr)
    }
    
    if let result = result {
        state.iikanjiResult = result
        return _strdup(result)
    }
    
    return nil
}

func requestIikanjiWithZenzai(keyword: IikanjiKeyword, context: String) -> String? {
    let state = IMEState.shared
    let zenzaiEnabled = state.getConfigValue("enable", default: false)
    guard zenzaiEnabled else {
        print("Zenzai is not enabled, falling back to OpenAI")
        state.iikanjiError = "Zenzaiが有効になっていません。設定でZenzaiを有効にするか、OpenAIプロバイダーを使用してください。"
        return nil
    }
    
    let promptContext = "\(context)\n\(keyword.zenzaiPrompt)"
    
    return state.sync {
        var tempComposingText = ComposingText()
        for char in promptContext {
            tempComposingText.insertAtCursorPosition(String(char), inputStyle: .roman2kana)
        }
        
        let options = state.getOptions(context: context)
        let converted = state.requestCandidates(tempComposingText, options: options)
        
        if let firstCandidate = converted.mainResults.first {
            return constructCandidateString(candidate: firstCandidate, hiragana: tempComposingText.convertTarget)
        }
        
        return nil
    }
}

func requestIikanjiWithOpenAI(keyword: IikanjiKeyword, context: String) -> String? {
    let state = IMEState.shared
    let apiKey = state.getConfigValue("openaiApiKey", default: "")
    guard !apiKey.isEmpty else {
        print("OpenAI API key is not set")
        state.iikanjiError = "OpenAI APIキーが設定されていません"
        return nil
    }
    
    let model = state.getConfigValue("openaiModel", default: "gpt-4o-mini")
    let maxTokens = state.getConfigValue("openaiMaxTokens", default: 256)
    let temperature = state.getConfigValue("openaiTemperature", default: 0.7)
    
    let isGpt5Series = model.hasPrefix("gpt-5")
    
    guard let url = URL(string: "https://api.openai.com/v1/chat/completions") else {
        return nil
    }
    
    var request = URLRequest(url: url)
    request.httpMethod = "POST"
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    request.timeoutInterval = 10.0
    
    var body: [String: Any] = [
        "model": model,
        "messages": [
            ["role": "system", "content": keyword.prompt],
            ["role": "user", "content": context]
        ],
        "temperature": temperature
    ]
    
    if isGpt5Series {
        body["max_completion_tokens"] = maxTokens
    } else {
        body["max_tokens"] = maxTokens
    }
    
    do {
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
    } catch {
        print("Failed to serialize request body: \(error)")
        return nil
    }
    
    nonisolated(unsafe) var result: String? = nil
    let semaphore = DispatchSemaphore(value: 0)
    
    let task = URLSession.shared.dataTask(with: request) { data, response, error in
        defer { semaphore.signal() }
        
        if let error = error {
            print("OpenAI API error: \(error)")
            return
        }
        
        guard let data = data else {
            print("No data received")
            return
        }
        
        do {
            if let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
               let choices = json["choices"] as? [[String: Any]],
               let firstChoice = choices.first,
               let message = firstChoice["message"] as? [String: Any],
               let content = message["content"] as? String {
                result = content.trimmingCharacters(in: .whitespacesAndNewlines)
            }
        } catch {
            print("Failed to parse response: \(error)")
        }
    }
    
    task.resume()
    _ = semaphore.wait(timeout: .now() + 10.0)
    
    return result
}
