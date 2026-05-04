package dev.stephyu.core.chat.adapter.out.nlp

import dev.stephyu.core.chat.adapter.out.nlp.dto.NlpAnalysisContextDto
import dev.stephyu.core.chat.adapter.out.nlp.dto.NlpAnalysisRequestDto
import dev.stephyu.core.chat.adapter.out.nlp.dto.NlpAnalysisResponseDto
import dev.stephyu.core.chat.adapter.out.nlp.dto.NlpContextSlotsDto
import dev.stephyu.core.chat.application.port.out.NlpAnalyzer
import dev.stephyu.core.chat.domain.intent.IntentName
import dev.stephyu.core.chat.domain.nlp.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse

class HttpNlpAnalyzer(
    private val config: NlpClientConfig,
    private val httpClient: HttpClient = HttpClient.newHttpClient(),
    private val json: Json = Json { ignoreUnknownKeys = true },
) : NlpAnalyzer {
    override suspend fun analyze(text: String, domain: String, context: NlpAnalysisContext?): NlpAnalysis =
        withContext(Dispatchers.IO) {
            val payload = NlpAnalysisRequestDto(
                text = text,
                domain = domain,
                context = context?.toDto(),
            )
            val request = HttpRequest.newBuilder()
                .uri(URI.create("${config.baseUrl.trimEnd('/')}/analyze"))
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(json.encodeToString(payload)))
                .build()

            runCatching {
                val response = httpClient.send(request, HttpResponse.BodyHandlers.ofString())
                if (response.statusCode() !in 200..299) {
                    return@runCatching NlpAnalysis.unavailable
                }
                json.decodeFromString<NlpAnalysisResponseDto>(response.body()).toDomain()
            }.getOrDefault(NlpAnalysis.unavailable)
        }

    private fun NlpAnalysisContext.toDto(): NlpAnalysisContextDto =
        NlpAnalysisContextDto(
            currentIntent = currentIntent?.wireName,
            previousIntent = previousIntent?.wireName,
            slotsFilled = NlpContextSlotsDto(
                date = slotsFilled[SlotName.DATE],
                time = slotsFilled[SlotName.TIME],
                people = slotsFilled[SlotName.PEOPLE],
                name = slotsFilled[SlotName.NAME],
                phone = slotsFilled[SlotName.PHONE],
                email = slotsFilled[SlotName.EMAIL],
                menuItem = slotsFilled[SlotName.MENU_ITEM],
                priceItem = slotsFilled[SlotName.PRICE_ITEM],
                location = slotsFilled[SlotName.LOCATION],
            ),
            requiredSlots = requiredSlots.map { it.wireName },
        )

    private fun NlpAnalysisResponseDto.toDomain(): NlpAnalysis =
        NlpAnalysis(
            intent = NlpIntent(
                name = IntentName.fromWireName(intent.name),
                confidence = intent.confidence,
                source = intent.source,
                alternatives = intent.alternatives.mapKeys { (wireName, _) -> IntentName.fromWireName(wireName) },
            ),
            intents = intents.ifEmpty {
                listOf(intent.toCandidate())
            }.map {
                NlpIntent(
                    name = IntentName.fromWireName(it.name),
                    confidence = it.confidence,
                    source = it.source,
                )
            },
            utterance = NlpUtterance(
                kind = NlpUtteranceKind.fromWireName(utterance.kind),
                confidence = utterance.confidence,
                source = utterance.source,
            ),
            entities = entities.map {
                NlpEntity(
                    type = SlotName.fromNlpWireName(it.type),
                    value = it.value,
                    confidence = it.confidence,
                    source = it.source,
                    rawValue = it.rawValue ?: it.value,
                    resolution = it.resolution,
                    normalizationStatus = NlpEntityNormalizationStatus.fromWireName(it.normalizationStatus),
                )
            },
            warnings = warnings,
        )

    private fun dev.stephyu.core.chat.adapter.out.nlp.dto.NlpIntentDto.toCandidate() =
        dev.stephyu.core.chat.adapter.out.nlp.dto.NlpIntentCandidateDto(
            name = name,
            confidence = confidence,
            source = source,
        )
}


