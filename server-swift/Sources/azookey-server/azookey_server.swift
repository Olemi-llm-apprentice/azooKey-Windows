import KanaKanjiConverterModule
import Foundation
import ffi

@MainActor let converter = KanaKanjiConverter()
@MainActor var composingText = ComposingText()

@MainActor var execURL = URL(filePath: "")
@MainActor var config: [String : Any] = [
    "enable": false,
    "profile": "",
    "learningEnabled": true,
    "predictionEnabled": true,
    "shouldResetMemory": false,
    // いい感じ変換設定
    "iikanjiEnabled": false,
    "iikanjiProvider": "openai",
    "iikanjiApiKey": "",
    "iikanjiModel": "gpt-4o-mini",
    "iikanjiMaxTokens": 256,
    "iikanjiTemperature": 0.7,
]

// いい感じ変換キーワード定義
enum IikanjiKeyword: String, CaseIterable {
    case eigo = "えいご"
    case nihongo = "にほんご"
    case emoji = "えもじ"
    case iikae = "いいかえ"
    case keigo = "けいご"
    case tamego = "ためご"
    case kousei = "こうせい"
    
    // カタカナバリアント
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

// いい感じ変換結果を保持
@MainActor var iikanjiResult: String? = nil
@MainActor var iikanjiError: String? = nil

// 学習データの保存先ディレクトリ
@MainActor var memoryURL: URL = {
    if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
        let url = URL(filePath: appDataPath).appendingPathComponent("Azookey/memory")
        // ディレクトリが存在しない場合は作成
        try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
    return URL(filePath: "./memory")
}()

// ユーザー辞書の保存先ディレクトリ
@MainActor var userDictionaryURL: URL = {
    if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
        let url = URL(filePath: appDataPath).appendingPathComponent("Azookey/user_dictionary")
        // ディレクトリが存在しない場合は作成
        try? FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
    return URL(filePath: "./user_dictionary")
}()

@MainActor func getOptions(context: String = "") -> ConvertRequestOptions {
    let learningEnabled = (config["learningEnabled"] as? Bool) ?? true
    let predictionEnabled = (config["predictionEnabled"] as? Bool) ?? true
    let shouldReset = (config["shouldResetMemory"] as? Bool) ?? false
    
    // リセットフラグが立っていたらクリア
    if shouldReset {
        config["shouldResetMemory"] = false
    }
    
    return ConvertRequestOptions(
        requireJapanesePrediction: predictionEnabled,
        requireEnglishPrediction: false,
        keyboardLanguage: .ja_JP,
        learningType: learningEnabled ? .inputAndOutput : .nothing,
        maxMemoryCount: 65536,
        shouldResetMemory: shouldReset,
        dictionaryResourceURL: execURL.appendingPathComponent("Dictionary"),
        memoryDirectoryURL: memoryURL,
        sharedContainerURL: userDictionaryURL,
        textReplacer: .init {
            return execURL.appendingPathComponent("EmojiDictionary").appendingPathComponent("emoji_all_E15.1.txt")
        },
        // zenzai
        zenzaiMode: config["enable"] as! Bool ? .on(
            weight: execURL.appendingPathComponent("zenz.gguf"),
            inferenceLimit: 1,
            requestRichCandidates: true,
            personalizationMode: nil,
            versionDependentMode: .v3(
                .init(
                    profile: config["profile"] as! String,
                    leftSideContext: context
                )
            )
        ) : .off,
        preloadDictionary: true,
        metadata: .init(versionString: "Azookey for Windows")
    )
}

class SimpleComposingText {
    init(text: String, cursor: Int) {
        self.text = UnsafeMutablePointer<CChar>(mutating: text.utf8String)!
        self.cursor = cursor
    }

    var text: UnsafeMutablePointer<CChar>
    var cursor: Int
}

struct SComposingText {
    var text: UnsafeMutablePointer<CChar>
    var cursor: Int
}

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

@_silgen_name("LoadConfig")
@MainActor public func load_config() {
    if let appDataPath = ProcessInfo.processInfo.environment["APPDATA"] {
        let settingsPath = URL(filePath: appDataPath).appendingPathComponent("Azookey/settings.json")
        
        do {
            let data = try Data(contentsOf: settingsPath)
            if let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] {
                // Zenzai設定の読み込み
                if let zenzaiDict = json["zenzai"] as? [String: Any] {
                    if let enableValue = zenzaiDict["enable"] as? Bool {
                        config["enable"] = enableValue
                    }
                    
                    if let profileValue = zenzaiDict["profile"] as? String {
                        config["profile"] = profileValue
                    }
                }
                
                // 学習設定の読み込み
                if let learningDict = json["learning"] as? [String: Any] {
                    if let enabledValue = learningDict["enabled"] as? Bool {
                        config["learningEnabled"] = enabledValue
                    }
                }
                
                // 予測変換設定の読み込み
                if let predictionDict = json["prediction"] as? [String: Any] {
                    if let enabledValue = predictionDict["enabled"] as? Bool {
                        config["predictionEnabled"] = enabledValue
                    }
                }
                
                // いい感じ変換設定の読み込み
                if let iikanjiDict = json["iikanji"] as? [String: Any] {
                    if let enabledValue = iikanjiDict["enabled"] as? Bool {
                        config["iikanjiEnabled"] = enabledValue
                    }
                    if let providerValue = iikanjiDict["provider"] as? String {
                        config["iikanjiProvider"] = providerValue
                    }
                    if let apiKeyValue = iikanjiDict["api_key"] as? String {
                        config["iikanjiApiKey"] = apiKeyValue
                    }
                    if let modelValue = iikanjiDict["model"] as? String {
                        config["iikanjiModel"] = modelValue
                    }
                    if let maxTokensValue = iikanjiDict["max_tokens"] as? Int {
                        config["iikanjiMaxTokens"] = maxTokensValue
                    }
                    if let temperatureValue = iikanjiDict["temperature"] as? Double {
                        config["iikanjiTemperature"] = temperatureValue
                    }
                }
            }
        } catch {
            print("Failed to read settings: \(error)")
        }
    }
}

// OpenAI API呼び出し（同期的に結果を取得）
@MainActor func requestIikanji(keyword: IikanjiKeyword, context: String) -> String? {
    let enabled = (config["iikanjiEnabled"] as? Bool) ?? false
    guard enabled else {
        print("Iikanji is disabled")
        return nil
    }
    
    let apiKey = (config["iikanjiApiKey"] as? String) ?? ""
    guard !apiKey.isEmpty else {
        print("Iikanji API key is not set")
        iikanjiError = "APIキーが設定されていません"
        return nil
    }
    
    let model = (config["iikanjiModel"] as? String) ?? "gpt-4o-mini"
    let maxTokens = (config["iikanjiMaxTokens"] as? Int) ?? 256
    let temperature = (config["iikanjiTemperature"] as? Double) ?? 0.7
    
    guard !context.isEmpty else {
        print("Context is empty")
        iikanjiError = "変換対象のテキストがありません"
        return nil
    }
    
    // URLリクエストを作成
    guard let url = URL(string: "https://api.openai.com/v1/chat/completions") else {
        return nil
    }
    
    var request = URLRequest(url: url)
    request.httpMethod = "POST"
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    request.timeoutInterval = 10.0
    
    let body: [String: Any] = [
        "model": model,
        "messages": [
            ["role": "system", "content": keyword.prompt],
            ["role": "user", "content": context]
        ],
        "max_tokens": maxTokens,
        "temperature": temperature
    ]
    
    do {
        request.httpBody = try JSONSerialization.data(withJSONObject: body)
    } catch {
        print("Failed to serialize request body: \(error)")
        return nil
    }
    
    // 同期的にリクエストを実行（セマフォを使用）
    var result: String? = nil
    let semaphore = DispatchSemaphore(value: 0)
    
    let task = URLSession.shared.dataTask(with: request) { data, response, error in
        defer { semaphore.signal() }
        
        if let error = error {
            print("Iikanji API error: \(error)")
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

// いい感じ変換を実行
@_silgen_name("RequestIikanji")
@MainActor public func request_iikanji(
    keywordPtr: UnsafePointer<CChar>,
    contextPtr: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>? {
    let keywordStr = String(cString: keywordPtr)
    let contextStr = String(cString: contextPtr)
    
    guard let keyword = IikanjiKeyword.detect(keywordStr) else {
        print("Unknown iikanji keyword: \(keywordStr)")
        return nil
    }
    
    if let result = requestIikanji(keyword: keyword, context: contextStr) {
        iikanjiResult = result
        return _strdup(result)
    }
    
    return nil
}

// いい感じ変換キーワードかどうかを判定
@_silgen_name("IsIikanjiKeyword")
@MainActor public func is_iikanji_keyword(
    inputPtr: UnsafePointer<CChar>
) -> Bool {
    let enabled = (config["iikanjiEnabled"] as? Bool) ?? false
    guard enabled else {
        return false
    }
    
    let input = String(cString: inputPtr)
    return IikanjiKeyword.detect(input) != nil
}

// いい感じ変換が有効かどうか
@_silgen_name("IsIikanjiEnabled")
@MainActor public func is_iikanji_enabled() -> Bool {
    return (config["iikanjiEnabled"] as? Bool) ?? false
}

@_silgen_name("ResetLearning")
@MainActor public func reset_learning() {
    // 次回の変換リクエスト時にリセットフラグを立てる
    config["shouldResetMemory"] = true
    
    // 即座にリセットを反映するためにダミーの変換を実行
    var tempComposingText = ComposingText()
    tempComposingText.insertAtCursorPosition("a", inputStyle: .roman2kana)
    _ = converter.requestCandidates(tempComposingText, options: getOptions())
    
    print("Learning data reset requested")
}

@_silgen_name("Initialize")
@MainActor public func initialize(
    path: UnsafePointer<CChar>,
    use_zenzai: Bool
) {
    let path = String(cString: path)
    execURL = URL(filePath: path)

    load_config()

    composingText.insertAtCursorPosition("a", inputStyle: .roman2kana)
    converter.requestCandidates(composingText, options: getOptions())
    composingText = ComposingText()
}

@_silgen_name("AppendText")
@MainActor public func append_text(
    input: UnsafePointer<CChar>,
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let inputString = String(cString: input)
    composingText.insertAtCursorPosition(inputString, inputStyle: .roman2kana)

    cursorPtr.pointee = composingText.convertTargetCursorPosition    
    return _strdup(composingText.convertTarget)!
}

@_silgen_name("RemoveText")
@MainActor public func remove_text(
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    composingText.deleteBackwardFromCursorPosition(count: 1)

    cursorPtr.pointee = composingText.convertTargetCursorPosition
    return _strdup(composingText.convertTarget)!
}

@_silgen_name("MoveCursor")
@MainActor public func move_cursor(
    offset: Int32,
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let previousCursor = composingText.convertTargetCursorPosition
    let cursor = composingText.moveCursorFromCursorPosition(count: Int(offset))
    print("offset: \(offset), cursor: \(cursor)")

    cursorPtr.pointee = cursor
    return _strdup(composingText.convertTarget)!
}

@_silgen_name("ClearText")
@MainActor public func clear_text() {
    composingText = ComposingText()
}

func to_list_pointer(_ list: [FFICandidate]) -> UnsafeMutablePointer<UnsafeMutablePointer<FFICandidate>?> {
    let pointer = UnsafeMutablePointer<UnsafeMutablePointer<FFICandidate>?>.allocate(capacity: list.count)
    for (i, item) in list.enumerated() {
        pointer[i] = UnsafeMutablePointer<FFICandidate>.allocate(capacity: 1)
        pointer[i]?.pointee = item
    }
    return pointer
}

@_silgen_name("GetComposedText")
@MainActor public func get_composed_text(lengthPtr: UnsafeMutablePointer<Int>) -> UnsafeMutablePointer<UnsafeMutablePointer<FFICandidate>?> {
    let hiragana = composingText.convertTarget
    let contextString = (config["context"] as? String) ?? ""
    let options = getOptions(context: contextString)
    let converted = converter.requestCandidates(composingText, options: options)
    var result: [FFICandidate] = []

    for i in 0..<converted.mainResults.count {
        let candidate = converted.mainResults[i]

        let text = strdup(constructCandidateString(candidate: candidate, hiragana: hiragana))
        let hiragana = strdup(hiragana)
        let correspondingCount = candidate.correspondingCount

        var afterComposingText = composingText
        afterComposingText.prefixComplete(correspondingCount: correspondingCount)
        let subtext = strdup(afterComposingText.convertTarget)

        result.append(FFICandidate(text: text, subtext: subtext, hiragana: hiragana, correspondingCount: Int32(correspondingCount)))        
    }

    lengthPtr.pointee = result.count

    return to_list_pointer(result)
}

@_silgen_name("ShrinkText")
@MainActor public func shrink_text(
    offset: Int32
) -> UnsafeMutablePointer<CChar>  {
    var afterComposingText = composingText
    afterComposingText.prefixComplete(correspondingCount: Int(offset))
    composingText = afterComposingText

    return _strdup(composingText.convertTarget)!
}

@_silgen_name("SetContext")
@MainActor public func set_context(
    context: UnsafePointer<CChar>
) {
    let contextString = String(cString: context)
    config["context"] = contextString
}